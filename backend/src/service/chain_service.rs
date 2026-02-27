use std::{path::Path, sync::Arc, time::Duration};

use base64::Engine;
use serde_json::json;
use solana_sdk::{
    hash::hashv, message::VersionedMessage, pubkey::Pubkey, transaction::VersionedTransaction,
    signature::{Keypair, Signer},
};
use tokio::time::{Instant, sleep};

use crate::{
    config::AppConfig,
    error::ApiError,
    module::chain::schema::{
        ChainContextResponse, ConfirmSignatureRequest, ConfirmSignatureResponse, DerivePdaRequest,
        DerivePdaResponse, SeedEncoding, SimulateTransactionRequest, SimulateTransactionResponse,
        SubmitAndConfirmRequest, SubmitAndConfirmResponse, SubmitTransactionRequest,
        SubmitTransactionResponse,
    },
    service::ops_service::OpsService,
};

#[derive(Clone)]
pub struct ChainService {
    rpc_url: String,
    commitment: String,
    cluster: String,
    program_id: String,
    idl_loaded: bool,
    anchor_idl_address: Option<String>,
    http: reqwest::Client,
    ops_service: Option<OpsService>,
    relayer: Option<Arc<Keypair>>,
}

impl ChainService {
    pub async fn new(
        config: &AppConfig,
        http: reqwest::Client,
        ops_service: Option<OpsService>,
    ) -> anyhow::Result<Self> {
        let chain = &config.chain;
        let idl_path = config
            .chain
            .idl_path
            .clone()
            .unwrap_or_else(|| "../smartcontract/app/idl/ticketing_core.json".to_string());

        let mut idl_loaded = false;
        let mut anchor_idl_address = None;
        let relayer = load_keypair(&chain.anchor_wallet).map(Arc::new);

        if Path::new(&idl_path).exists() {
            if let Ok(raw) = tokio::fs::read_to_string(&idl_path).await {
                if let Ok(value) = serde_json::from_str::<serde_json::Value>(&raw) {
                    if let Some(address) = value.get("address").and_then(|v| v.as_str()) {
                        idl_loaded = true;
                        anchor_idl_address = Some(address.to_string());
                        if address != chain.program_id {
                            anyhow::bail!(
                                "anchor idl address mismatch: env PROGRAM_ID={} idl address={}",
                                chain.program_id,
                                address
                            );
                        }
                    }
                }
            }
        }

        Ok(Self {
            rpc_url: chain.rpc_url.clone(),
            commitment: chain.commitment.clone(),
            cluster: chain.cluster.clone(),
            program_id: chain.program_id.clone(),
            idl_loaded,
            anchor_idl_address,
            http,
            ops_service,
            relayer,
        })
    }

    pub fn context(&self) -> ChainContextResponse {
        ChainContextResponse {
            cluster: self.cluster.clone(),
            rpc_url: self.rpc_url.clone(),
            commitment: self.commitment.clone(),
            program_id: self.program_id.clone(),
            anchor_idl_address: self.anchor_idl_address.clone(),
            idl_loaded: self.idl_loaded,
        }
    }

    pub fn derive_pda(&self, payload: DerivePdaRequest) -> Result<DerivePdaResponse, ApiError> {
        let program = self
            .program_id
            .parse::<Pubkey>()
            .map_err(|_| ApiError::BadRequest("invalid PROGRAM_ID".to_string()))?;

        let seed_bytes: Result<Vec<Vec<u8>>, ApiError> = payload
            .seeds
            .into_iter()
            .map(|seed| decode_seed(seed.encoding, seed.value))
            .collect();

        let seeds = seed_bytes?;
        let seed_slices: Vec<&[u8]> = seeds.iter().map(|v| v.as_slice()).collect();

        let (pda, bump) = Pubkey::find_program_address(&seed_slices, &program);
        Ok(DerivePdaResponse {
            pda: pda.to_string(),
            bump,
        })
    }

    pub async fn simulate_transaction(
        &self,
        payload: SimulateTransactionRequest,
    ) -> Result<SimulateTransactionResponse, ApiError> {
        let params = json!([
            payload.transaction_base64,
            {
                "encoding": "base64",
                "sigVerify": payload.sig_verify,
                "replaceRecentBlockhash": payload.replace_recent_blockhash,
                "commitment": self.commitment,
            }
        ]);

        let value = self.rpc_call("simulateTransaction", params).await?;
        let logs = value
            .get("logs")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|x| x.as_str().map(ToString::to_string))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        Ok(SimulateTransactionResponse {
            err: value.get("err").cloned(),
            logs,
            units_consumed: value.get("unitsConsumed").and_then(|v| v.as_u64()),
        })
    }

    pub async fn simulate_transaction_program_ix(
        &self,
        payload: SimulateTransactionRequest,
        expected_instructions: &[&str],
    ) -> Result<SimulateTransactionResponse, ApiError> {
        self.validate_program_instruction(&payload.transaction_base64, expected_instructions)?;
        self.simulate_transaction(payload).await
    }

    pub async fn submit_transaction(
        &self,
        payload: SubmitTransactionRequest,
    ) -> Result<SubmitTransactionResponse, ApiError> {
        let params = json!([
            payload.transaction_base64,
            {
                "encoding": "base64",
                "skipPreflight": payload.skip_preflight,
                "preflightCommitment": self.commitment,
                "maxRetries": payload.max_retries,
            }
        ]);

        let value = self.rpc_call("sendTransaction", params).await?;
        let signature = value
            .as_str()
            .ok_or_else(|| ApiError::map_chain_error("missing signature"))?;

        Ok(SubmitTransactionResponse {
            signature: signature.to_string(),
        })
    }

    pub async fn confirm_signature(
        &self,
        payload: ConfirmSignatureRequest,
    ) -> Result<ConfirmSignatureResponse, ApiError> {
        let deadline = Instant::now() + Duration::from_millis(payload.timeout_ms);

        while Instant::now() < deadline {
            let status = self.fetch_signature_status(&payload.signature).await?;

            if let Some(entry) = status {
                let confirmation_status = entry
                    .get("confirmationStatus")
                    .and_then(|v| v.as_str())
                    .map(ToString::to_string);
                let err = entry.get("err").cloned().filter(|value| !value.is_null());

                if err.is_some() {
                    return Ok(ConfirmSignatureResponse {
                        confirmed: false,
                        confirmation_status,
                        err,
                    });
                }

                if matches!(
                    confirmation_status.as_deref(),
                    Some("confirmed") | Some("finalized")
                ) {
                    return Ok(ConfirmSignatureResponse {
                        confirmed: true,
                        confirmation_status,
                        err: None,
                    });
                }
            }

            sleep(Duration::from_millis(payload.poll_ms)).await;
        }

        Ok(ConfirmSignatureResponse {
            confirmed: false,
            confirmation_status: Some("timeout".to_string()),
            err: None,
        })
    }

    pub async fn submit_and_confirm(
        &self,
        payload: SubmitAndConfirmRequest,
    ) -> Result<SubmitAndConfirmResponse, ApiError> {
        let started = Instant::now();
        let result = async {
            let submit = self
                .submit_transaction(SubmitTransactionRequest {
                    transaction_base64: payload.transaction_base64,
                    skip_preflight: payload.skip_preflight,
                    max_retries: payload.max_retries,
                })
                .await?;

            let confirm = self
                .confirm_signature(ConfirmSignatureRequest {
                    signature: submit.signature.clone(),
                    timeout_ms: payload.timeout_ms,
                    poll_ms: payload.poll_ms,
                })
                .await?;

            if !confirm.confirmed {
                return Err(ApiError::map_chain_error("transaction not confirmed"));
            }

            Ok(SubmitAndConfirmResponse {
                signature: submit.signature,
                confirmation_status: confirm.confirmation_status,
            })
        }
        .await;

        if let Some(ops) = &self.ops_service {
            ops.record_tx_result(result.is_ok(), Some(started.elapsed().as_millis()))
                .await;
        }

        result
    }

    pub async fn submit_and_confirm_program_ix(
        &self,
        payload: SubmitAndConfirmRequest,
        expected_instructions: &[&str],
    ) -> Result<SubmitAndConfirmResponse, ApiError> {
        if let Err(err) =
            self.validate_program_instruction(&payload.transaction_base64, expected_instructions)
        {
            if let Some(ops) = &self.ops_service {
                ops.record_tx_result(false, None).await;
            }
            return Err(err);
        }
        self.submit_and_confirm(payload).await
    }

    pub async fn cosign_relayer_and_submit(
        &self,
        payload: SubmitAndConfirmRequest,
        expected_instructions: &[&str],
    ) -> Result<SubmitAndConfirmResponse, ApiError> {
        self.validate_program_instruction(&payload.transaction_base64, expected_instructions)?;
        let relayer = self
            .relayer
            .as_ref()
            .ok_or_else(|| ApiError::BadRequest("relayer signer is not configured".to_string()))?;

        let raw = base64::engine::general_purpose::STANDARD
            .decode(&payload.transaction_base64)
            .map_err(|_| ApiError::BadRequest("invalid transaction_base64".to_string()))?;
        let mut tx: VersionedTransaction = bincode::deserialize(&raw)
            .map_err(|_| ApiError::BadRequest("invalid serialized transaction".to_string()))?;

        let signer_index = signer_index_for(&tx.message, &relayer.pubkey()).ok_or_else(|| {
            ApiError::BadRequest("transaction does not include relayer as signer".to_string())
        })?;

        let message_bytes = tx.message.serialize();
        let relayer_sig = relayer.sign_message(&message_bytes);

        if signer_index >= tx.signatures.len() {
            return Err(ApiError::BadRequest(
                "invalid signature layout for transaction".to_string(),
            ));
        }

        tx.signatures[signer_index] = relayer_sig;
        let final_tx_base64 = base64::engine::general_purpose::STANDARD.encode(
            bincode::serialize(&tx).map_err(ApiError::map_chain_error)?,
        );

        let submit = self
            .submit_transaction(SubmitTransactionRequest {
                transaction_base64: final_tx_base64,
                skip_preflight: payload.skip_preflight,
                max_retries: payload.max_retries,
            })
            .await?;

        let confirm = self
            .confirm_signature(ConfirmSignatureRequest {
                signature: submit.signature.clone(),
                timeout_ms: payload.timeout_ms,
                poll_ms: payload.poll_ms,
            })
            .await?;

        if let Some(err) = confirm.err.filter(|value| !value.is_null()) {
            return Err(ApiError::map_chain_error(err));
        }

        Ok(SubmitAndConfirmResponse {
            signature: submit.signature,
            confirmation_status: confirm.confirmation_status,
        })
    }

    fn validate_program_instruction(
        &self,
        transaction_base64: &str,
        expected_instructions: &[&str],
    ) -> Result<(), ApiError> {
        let raw = base64::engine::general_purpose::STANDARD
            .decode(transaction_base64)
            .map_err(|_| ApiError::BadRequest("invalid transaction_base64".to_string()))?;
        let tx: VersionedTransaction = bincode::deserialize(&raw)
            .map_err(|_| ApiError::BadRequest("invalid serialized transaction".to_string()))?;
        let program_id = self
            .program_id
            .parse::<Pubkey>()
            .map_err(|_| ApiError::BadRequest("invalid PROGRAM_ID".to_string()))?;
        let expected_discriminators = expected_instructions
            .iter()
            .map(|name| anchor_discriminator(name))
            .collect::<Vec<_>>();

        let (keys, instructions) = match &tx.message {
            VersionedMessage::Legacy(msg) => (&msg.account_keys, &msg.instructions),
            VersionedMessage::V0(msg) => (&msg.account_keys, &msg.instructions),
        };

        for ix in instructions {
            let Some(pid) = keys.get(ix.program_id_index as usize) else {
                continue;
            };
            if pid != &program_id {
                continue;
            }

            if expected_discriminators.is_empty() {
                return Ok(());
            }

            if ix.data.len() < 8 {
                continue;
            }

            for expected in &expected_discriminators {
                if ix.data[..8] == expected[..] {
                    return Ok(());
                }
            }
        }

        Err(ApiError::BadRequest(format!(
            "transaction must target PROGRAM_ID and include one of instructions: {}",
            expected_instructions.join(", ")
        )))
    }

    async fn fetch_signature_status(
        &self,
        signature: &str,
    ) -> Result<Option<serde_json::Value>, ApiError> {
        let params = json!([[signature], { "searchTransactionHistory": true }]);
        let value = self.rpc_call("getSignatureStatuses", params).await?;

        let entry = value
            .get("value")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .cloned()
            .unwrap_or(serde_json::Value::Null);

        if entry.is_null() {
            Ok(None)
        } else {
            Ok(Some(entry))
        }
    }

    async fn rpc_call(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, ApiError> {
        let payload = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params,
        });

        let response = self
            .http
            .post(&self.rpc_url)
            .json(&payload)
            .send()
            .await
            .map_err(ApiError::map_chain_error)?;

        let body = response
            .json::<serde_json::Value>()
            .await
            .map_err(ApiError::map_chain_error)?;

        if let Some(err) = body.get("error") {
            return Err(ApiError::map_chain_error(err));
        }

        body.get("result")
            .cloned()
            .ok_or_else(|| ApiError::map_chain_error("missing rpc result"))
    }
}

fn decode_seed(encoding: SeedEncoding, value: String) -> Result<Vec<u8>, ApiError> {
    match encoding {
        SeedEncoding::Utf8 => Ok(value.into_bytes()),
        SeedEncoding::Hex => {
            hex::decode(value).map_err(|_| ApiError::BadRequest("invalid hex seed".to_string()))
        }
        SeedEncoding::Base58 => bs58::decode(value)
            .into_vec()
            .map_err(|_| ApiError::BadRequest("invalid base58 seed".to_string())),
    }
}

fn anchor_discriminator(name: &str) -> [u8; 8] {
    let preimage = format!("global:{name}");
    let hash = hashv(&[preimage.as_bytes()]).to_bytes();
    let mut out = [0u8; 8];
    out.copy_from_slice(&hash[..8]);
    out
}

fn load_keypair(path: &str) -> Option<Keypair> {
    let raw = std::fs::read_to_string(path).ok()?;
    let nums = serde_json::from_str::<Vec<u8>>(&raw).ok()?;
    Keypair::try_from(nums.as_slice()).ok()
}

fn signer_index_for(message: &VersionedMessage, signer: &Pubkey) -> Option<usize> {
    let (keys, required) = match message {
        VersionedMessage::Legacy(msg) => {
            (&msg.account_keys, msg.header.num_required_signatures as usize)
        }
        VersionedMessage::V0(msg) => (&msg.account_keys, msg.header.num_required_signatures as usize),
    };

    keys.iter()
        .take(required)
        .position(|pk| pk == signer)
}
