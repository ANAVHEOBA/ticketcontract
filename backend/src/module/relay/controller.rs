use axum::{Json, extract::State};

use crate::{
    app::AppState,
    error::AppResult,
    module::{
        auth::{
            guard::{AuthContext, require_any_role},
            model::Role,
        },
        chain::schema::SubmitAndConfirmRequest,
        relay::schema::{RelaySubmitRequest, RelaySubmitResponse},
    },
};

pub async fn submit(
    auth: AuthContext,
    State(state): State<AppState>,
    Json(payload): Json<RelaySubmitRequest>,
) -> AppResult<Json<RelaySubmitResponse>> {
    require_any_role(
        &auth,
        &[
            Role::ProtocolAdmin,
            Role::OrganizerAdmin,
            Role::Operator,
            Role::Financier,
        ],
    )?;

    let expected = payload
        .expected_instructions
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();

    let result = state
        .chain_service
        .cosign_relayer_and_submit(
            SubmitAndConfirmRequest {
                transaction_base64: payload.transaction_base64,
                skip_preflight: payload.skip_preflight,
                max_retries: payload.max_retries,
                timeout_ms: payload.timeout_ms,
                poll_ms: payload.poll_ms,
            },
            &expected,
        )
        .await?;

    Ok(Json(RelaySubmitResponse {
        signature: result.signature,
        confirmation_status: result.confirmation_status,
    }))
}
