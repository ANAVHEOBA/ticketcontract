use axum::{Json, extract::State};
use serde_json::json;

use crate::{
    app::AppState,
    error::AppResult,
    module::{
        auth::{
            guard::{AuthContext, require_any_role},
            model::Role,
        },
        chain::schema::{SimulateTransactionRequest, SubmitAndConfirmRequest},
        protocol_admin::schema::{
            ProtocolAdminActionResponse, ProtocolAdminSimRequest, ProtocolAdminSimResponse,
            ProtocolAdminTxRequest,
        },
    },
};

macro_rules! action_handlers {
    ($(($tx_fn:ident, $sim_fn:ident, $action:literal)),* $(,)?) => {
        $(
            pub async fn $tx_fn(
                auth: AuthContext,
                State(state): State<AppState>,
                Json(payload): Json<ProtocolAdminTxRequest>,
            ) -> AppResult<Json<ProtocolAdminActionResponse>> {
                require_protocol_admin(&auth)?;
                let result = state
                    .chain_service
                    .submit_and_confirm_program_ix(SubmitAndConfirmRequest {
                        transaction_base64: payload.transaction_base64,
                        skip_preflight: payload.skip_preflight,
                        max_retries: payload.max_retries,
                        timeout_ms: payload.timeout_ms,
                        poll_ms: payload.poll_ms,
                    }, &[$action])
                    .await?;

                if $action == "pause_protocol" {
                    state.ops_service.set_protocol_paused(true).await;
                } else if $action == "initialize_protocol" {
                    state.ops_service.set_protocol_paused(false).await;
                }

                let _ = state
                    .ops_service
                    .audit(
                        &auth.wallet,
                        role_name(&auth.role),
                        $action,
                        Some(json!({
                            "skip_preflight": payload.skip_preflight,
                            "max_retries": payload.max_retries
                        })),
                    )
                    .await;

                Ok(Json(ProtocolAdminActionResponse {
                    action: $action,
                    signature: result.signature,
                    confirmation_status: result.confirmation_status,
                }))
            }

            pub async fn $sim_fn(
                auth: AuthContext,
                State(state): State<AppState>,
                Json(payload): Json<ProtocolAdminSimRequest>,
            ) -> AppResult<Json<ProtocolAdminSimResponse>> {
                require_protocol_admin(&auth)?;
                let result = state
                    .chain_service
                    .simulate_transaction_program_ix(SimulateTransactionRequest {
                        transaction_base64: payload.transaction_base64,
                        sig_verify: payload.sig_verify,
                        replace_recent_blockhash: payload.replace_recent_blockhash,
                    }, &[$action])
                    .await?;

                let _ = state
                    .ops_service
                    .audit(
                        &auth.wallet,
                        role_name(&auth.role),
                        concat!($action, "_simulate"),
                        Some(json!({
                            "sig_verify": payload.sig_verify
                        })),
                    )
                    .await;

                Ok(Json(ProtocolAdminSimResponse {
                    action: $action,
                    err: result.err,
                    logs: result.logs,
                    units_consumed: result.units_consumed,
                }))
            }
        )*
    };
}

action_handlers!(
    (
        initialize_protocol,
        simulate_initialize_protocol,
        "initialize_protocol"
    ),
    (pause_protocol, simulate_pause_protocol, "pause_protocol"),
    (
        set_protocol_config,
        simulate_set_protocol_config,
        "set_protocol_config"
    ),
    (
        register_protocol_vaults,
        simulate_register_protocol_vaults,
        "register_protocol_vaults"
    ),
    (
        set_protocol_authorities,
        simulate_set_protocol_authorities,
        "set_protocol_authorities"
    ),
    (
        set_multisig_config,
        simulate_set_multisig_config,
        "set_multisig_config"
    ),
    (
        set_timelock_delay,
        simulate_set_timelock_delay,
        "set_timelock_delay"
    ),
    (
        queue_protocol_config_change,
        simulate_queue_protocol_config_change,
        "queue_protocol_config_change"
    ),
    (
        execute_protocol_config_change,
        simulate_execute_protocol_config_change,
        "execute_protocol_config_change"
    ),
    (
        begin_upgrade_authority_handoff,
        simulate_begin_upgrade_authority_handoff,
        "begin_upgrade_authority_handoff"
    ),
    (
        accept_upgrade_authority_handoff,
        simulate_accept_upgrade_authority_handoff,
        "accept_upgrade_authority_handoff"
    ),
    (
        emergency_rotate_admin,
        simulate_emergency_rotate_admin,
        "emergency_rotate_admin"
    ),
    (
        set_global_loyalty_multiplier,
        simulate_set_global_loyalty_multiplier,
        "set_global_loyalty_multiplier"
    ),
    (grant_role, simulate_grant_role, "grant_role"),
    (revoke_role, simulate_revoke_role, "revoke_role"),
    (
        rotate_authority,
        simulate_rotate_authority,
        "rotate_authority"
    ),
    (
        initialize_vault,
        simulate_initialize_vault,
        "initialize_vault"
    ),
    (snapshot_vault, simulate_snapshot_vault, "snapshot_vault"),
    (withdraw_vault, simulate_withdraw_vault, "withdraw_vault"),
    (
        upsert_registry_entry,
        simulate_upsert_registry_entry,
        "upsert_registry_entry"
    ),
);

fn require_protocol_admin(auth: &AuthContext) -> Result<(), crate::error::ApiError> {
    require_any_role(auth, &[Role::ProtocolAdmin])
}

fn role_name(role: &Role) -> &'static str {
    match role {
        Role::ProtocolAdmin => "protocol_admin",
        Role::OrganizerAdmin => "organizer_admin",
        Role::Operator => "operator",
        Role::Scanner => "scanner",
        Role::Financier => "financier",
    }
}
