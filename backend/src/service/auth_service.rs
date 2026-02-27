use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Context;
use base64::Engine;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use reqwest::StatusCode;
use serde::Deserialize;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};

use crate::{
    config::AppConfig,
    error::ApiError,
    module::auth::{
        crud::{AccessStore, NonceStore},
        model::{AccessProfile, AccessRule, JwtClaims, NonceRecord, Role},
        schema::{NonceResponse, VerifyRequest, VerifyResponse},
    },
};

#[derive(Clone, Default)]
pub struct AuthService {
    nonce_store: NonceStore,
    access_store: AccessStore,
    jwt_encoding_key: Option<EncodingKey>,
    jwt_decoding_key: Option<DecodingKey>,
    jwt_expires_seconds: u64,
    nonce_ttl_seconds: u64,
    google_client_id: Option<String>,
    allow_unlisted_wallets: bool,
    http: reqwest::Client,
}

impl AuthService {
    pub async fn new(config: &AppConfig, http: reqwest::Client) -> anyhow::Result<Self> {
        let nonce_store = NonceStore::default();
        let access_store = AccessStore::default();

        if let Some(raw) = &config.auth.access_rules_json {
            let rules = serde_json::from_str::<Vec<AccessRule>>(raw)
                .context("AUTH_ACCESS_RULES_JSON must be valid JSON array")?;
            access_store.seed_rules(&rules).await;
        }

        Ok(Self {
            nonce_store,
            access_store,
            jwt_encoding_key: Some(EncodingKey::from_secret(config.auth.jwt_secret.as_bytes())),
            jwt_decoding_key: Some(DecodingKey::from_secret(config.auth.jwt_secret.as_bytes())),
            jwt_expires_seconds: config.auth.jwt_expires_seconds,
            nonce_ttl_seconds: config.auth.siws_nonce_ttl_seconds,
            google_client_id: config.auth.google_client_id.clone(),
            allow_unlisted_wallets: config.auth.allow_unlisted_wallets,
            http,
        })
    }

    pub async fn issue_nonce(&self, wallet: String) -> Result<NonceResponse, ApiError> {
        validate_wallet(&wallet)?;

        let now = epoch_now();
        let expires_at = now + self.nonce_ttl_seconds;
        let nonce = uuid::Uuid::new_v4().simple().to_string();
        let message = format!(
            "Sign in to Ticketing Platform\\nWallet: {wallet}\\nNonce: {nonce}\\nIssued At: {now}\\nExpires At: {expires_at}"
        );

        self.nonce_store
            .upsert(
                wallet.clone(),
                NonceRecord {
                    nonce: nonce.clone(),
                    message: message.clone(),
                    expires_at_epoch: expires_at,
                },
            )
            .await;

        Ok(NonceResponse {
            wallet,
            nonce,
            message,
            expires_at_epoch: expires_at,
        })
    }

    pub async fn verify_and_issue_token(
        &self,
        request: VerifyRequest,
    ) -> Result<VerifyResponse, ApiError> {
        self.verify_wallet_nonce_signature(&request.wallet, &request.nonce, &request.signature)
            .await?;
        let Some(profile) = self.resolve_profile(&request.wallet).await else {
            return Err(ApiError::Forbidden);
        };
        self.issue_jwt(request.wallet, profile).await
    }

    pub async fn verify_provider_and_issue_token(
        &self,
        request: crate::module::auth::schema::ProviderVerifyRequest,
    ) -> Result<VerifyResponse, ApiError> {
        if request.provider.to_lowercase() != "google" {
            return Err(ApiError::BadRequest(
                "unsupported provider. currently supported: google".to_string(),
            ));
        }

        self.verify_google_id_token(&request.id_token).await?;
        self.verify_wallet_nonce_signature(&request.wallet, &request.nonce, &request.signature)
            .await?;

        let Some(profile) = self.resolve_profile(&request.wallet).await else {
            return Err(ApiError::Forbidden);
        };

        self.issue_jwt(request.wallet, profile).await
    }

    async fn resolve_profile(&self, wallet: &str) -> Option<AccessProfile> {
        if let Some(profile) = self.access_store.get(wallet).await {
            return Some(profile);
        }

        if !self.allow_unlisted_wallets {
            return None;
        }

        let fallback = AccessProfile {
            wallet: wallet.to_string(),
            role: Role::OrganizerAdmin,
            organizer_scopes: vec!["*".to_string()],
        };
        self.access_store.upsert(fallback.clone()).await;
        Some(fallback)
    }

    async fn issue_jwt(
        &self,
        wallet: String,
        profile: crate::module::auth::model::AccessProfile,
    ) -> Result<VerifyResponse, ApiError> {
        
        let now = epoch_now();
        let claims = JwtClaims {
            sub: wallet,
            role: profile.role.clone(),
            organizer_scopes: profile.organizer_scopes.clone(),
            iat: now,
            exp: now + self.jwt_expires_seconds,
        };

        let token = encode(
            &Header::default(),
            &claims,
            self.jwt_encoding_key.as_ref().ok_or(ApiError::Internal)?,
        )
        .map_err(|_| ApiError::Internal)?;

        Ok(VerifyResponse {
            access_token: token,
            token_type: "Bearer",
            expires_in: self.jwt_expires_seconds,
            role: profile.role,
            organizer_scopes: profile.organizer_scopes,
        })
    }

    async fn verify_wallet_nonce_signature(
        &self,
        wallet: &str,
        nonce: &str,
        signature: &str,
    ) -> Result<(), ApiError> {
        validate_wallet(wallet)?;
        let Some(record) = self.nonce_store.take(wallet).await else {
            return Err(ApiError::BadRequest(
                "nonce not found or already used".to_string(),
            ));
        };

        if record.nonce != nonce {
            return Err(ApiError::BadRequest("invalid nonce".to_string()));
        }

        if epoch_now() > record.expires_at_epoch {
            return Err(ApiError::BadRequest("nonce expired".to_string()));
        }

        verify_signature(wallet, &record.message, signature)
    }

    async fn verify_google_id_token(&self, id_token: &str) -> Result<(), ApiError> {
        let client_id = self.google_client_id.as_ref().ok_or_else(|| {
            ApiError::BadRequest("google auth is not configured on backend".to_string())
        })?;

        let response = self
            .http
            .get("https://oauth2.googleapis.com/tokeninfo")
            .query(&[("id_token", id_token)])
            .send()
            .await
            .map_err(|_| ApiError::Unauthorized)?;

        if response.status() != StatusCode::OK {
            return Err(ApiError::Unauthorized);
        }

        let body: GoogleTokenInfo = response.json().await.map_err(|_| ApiError::Unauthorized)?;
        if body.aud != *client_id {
            return Err(ApiError::Unauthorized);
        }
        if body.iss != "https://accounts.google.com" && body.iss != "accounts.google.com" {
            return Err(ApiError::Unauthorized);
        }
        let exp = body
            .exp
            .parse::<u64>()
            .map_err(|_| ApiError::Unauthorized)?;
        if exp <= epoch_now() {
            return Err(ApiError::Unauthorized);
        }
        Ok(())
    }

    pub fn decode_token(&self, token: &str) -> Result<JwtClaims, ApiError> {
        let data = decode::<JwtClaims>(
            token,
            self.jwt_decoding_key.as_ref().ok_or(ApiError::Internal)?,
            &Validation::default(),
        )
        .map_err(|_| ApiError::Unauthorized)?;

        Ok(data.claims)
    }
}

#[derive(Debug, Deserialize)]
struct GoogleTokenInfo {
    aud: String,
    iss: String,
    exp: String,
}

fn epoch_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn validate_wallet(wallet: &str) -> Result<(), ApiError> {
    let bytes = bs58::decode(wallet)
        .into_vec()
        .map_err(|_| ApiError::BadRequest("wallet must be base58".to_string()))?;

    if bytes.len() != 32 {
        return Err(ApiError::BadRequest(
            "wallet must decode to 32 bytes".to_string(),
        ));
    }

    Ok(())
}

fn verify_signature(wallet: &str, message: &str, signature_text: &str) -> Result<(), ApiError> {
    let wallet_bytes = bs58::decode(wallet)
        .into_vec()
        .map_err(|_| ApiError::BadRequest("invalid wallet".to_string()))?;
    let wallet_array: [u8; 32] = wallet_bytes
        .try_into()
        .map_err(|_| ApiError::BadRequest("wallet must be 32 bytes".to_string()))?;

    let signature_bytes = decode_signature(signature_text)?;
    let signature = Signature::from_slice(&signature_bytes)
        .map_err(|_| ApiError::BadRequest("invalid signature".to_string()))?;

    let verifying_key = VerifyingKey::from_bytes(&wallet_array)
        .map_err(|_| ApiError::BadRequest("invalid wallet key".to_string()))?;

    verifying_key
        .verify(message.as_bytes(), &signature)
        .map_err(|_| ApiError::Unauthorized)
}

fn decode_signature(signature_text: &str) -> Result<[u8; 64], ApiError> {
    if let Ok(bytes) = bs58::decode(signature_text).into_vec() {
        if let Ok(array) = bytes.try_into() {
            return Ok(array);
        }
    }

    let bytes = base64::engine::general_purpose::STANDARD
        .decode(signature_text)
        .map_err(|_| ApiError::BadRequest("signature must be base58 or base64".to_string()))?;

    bytes
        .try_into()
        .map_err(|_| ApiError::BadRequest("signature must decode to 64 bytes".to_string()))
}
