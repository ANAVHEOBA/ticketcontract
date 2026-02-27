use axum::{
    Json,
    extract::{Query, State},
};

use crate::{
    app::AppState,
    error::{ApiError, AppResult},
    module::{
        auth::{
            guard::{AuthContext, require_any_role, require_organizer_scope},
            model::Role,
        },
        chain::schema::SubmitAndConfirmRequest,
        loyalty_trust::{
            crud,
            schema::{
                LoyaltyQuery, LoyaltyReadResponse, LoyaltyTrustActionResponse, LoyaltyTxRequest,
                TrustSchemaTxRequest, TrustSignalQuery, TrustSignalReadResponse,
                TrustSignalTxRequest,
            },
        },
    },
};

pub async fn accrue_points(
    auth: AuthContext,
    State(state): State<AppState>,
    Json(payload): Json<LoyaltyTxRequest>,
) -> AppResult<Json<LoyaltyTrustActionResponse>> {
    require_loyalty_writer(&auth, &payload.organizer_id)?;
    let result = submit(
        &state,
        "accrue_points",
        &payload.transaction_base64,
        payload.skip_preflight,
        payload.max_retries,
        payload.timeout_ms,
        payload.poll_ms,
    )
    .await?;

    Ok(Json(LoyaltyTrustActionResponse {
        action: "accrue_points",
        organizer_id: Some(payload.organizer_id),
        event_id: Some(payload.event_id),
        wallet: Some(payload.wallet),
        signal_id: None,
        signature: result.0,
        confirmation_status: result.1,
    }))
}

pub async fn redeem_points(
    auth: AuthContext,
    State(state): State<AppState>,
    Json(payload): Json<LoyaltyTxRequest>,
) -> AppResult<Json<LoyaltyTrustActionResponse>> {
    require_loyalty_writer(&auth, &payload.organizer_id)?;
    let result = submit(
        &state,
        "redeem_points",
        &payload.transaction_base64,
        payload.skip_preflight,
        payload.max_retries,
        payload.timeout_ms,
        payload.poll_ms,
    )
    .await?;

    Ok(Json(LoyaltyTrustActionResponse {
        action: "redeem_points",
        organizer_id: Some(payload.organizer_id),
        event_id: Some(payload.event_id),
        wallet: Some(payload.wallet),
        signal_id: None,
        signature: result.0,
        confirmation_status: result.1,
    }))
}

pub async fn record_purchase_signal(
    auth: AuthContext,
    State(state): State<AppState>,
    Json(payload): Json<TrustSignalTxRequest>,
) -> AppResult<Json<LoyaltyTrustActionResponse>> {
    require_signal_writer(&auth, &payload.organizer_id)?;
    let result = submit(
        &state,
        "record_purchase_input",
        &payload.transaction_base64,
        payload.skip_preflight,
        payload.max_retries,
        payload.timeout_ms,
        payload.poll_ms,
    )
    .await?;

    Ok(Json(LoyaltyTrustActionResponse {
        action: "record_purchase_signal",
        organizer_id: Some(payload.organizer_id),
        event_id: Some(payload.event_id),
        wallet: Some(payload.wallet),
        signal_id: payload.signal_id,
        signature: result.0,
        confirmation_status: result.1,
    }))
}

pub async fn record_attendance_signal(
    auth: AuthContext,
    State(state): State<AppState>,
    Json(payload): Json<TrustSignalTxRequest>,
) -> AppResult<Json<LoyaltyTrustActionResponse>> {
    require_signal_writer(&auth, &payload.organizer_id)?;
    let result = submit(
        &state,
        "record_attendance_input",
        &payload.transaction_base64,
        payload.skip_preflight,
        payload.max_retries,
        payload.timeout_ms,
        payload.poll_ms,
    )
    .await?;

    Ok(Json(LoyaltyTrustActionResponse {
        action: "record_attendance_signal",
        organizer_id: Some(payload.organizer_id),
        event_id: Some(payload.event_id),
        wallet: Some(payload.wallet),
        signal_id: payload.signal_id,
        signature: result.0,
        confirmation_status: result.1,
    }))
}

pub async fn flag_trust_abuse(
    auth: AuthContext,
    State(state): State<AppState>,
    Json(payload): Json<TrustSignalTxRequest>,
) -> AppResult<Json<LoyaltyTrustActionResponse>> {
    require_signal_writer(&auth, &payload.organizer_id)?;
    let result = submit(
        &state,
        "flag_trust_abuse",
        &payload.transaction_base64,
        payload.skip_preflight,
        payload.max_retries,
        payload.timeout_ms,
        payload.poll_ms,
    )
    .await?;

    Ok(Json(LoyaltyTrustActionResponse {
        action: "flag_trust_abuse",
        organizer_id: Some(payload.organizer_id),
        event_id: Some(payload.event_id),
        wallet: Some(payload.wallet),
        signal_id: payload.signal_id,
        signature: result.0,
        confirmation_status: result.1,
    }))
}

pub async fn set_trust_schema_version(
    auth: AuthContext,
    State(state): State<AppState>,
    Json(payload): Json<TrustSchemaTxRequest>,
) -> AppResult<Json<LoyaltyTrustActionResponse>> {
    require_any_role(&auth, &[Role::ProtocolAdmin])?;
    if let Some(org) = payload.organizer_id.as_deref() {
        require_organizer_scope(&auth, org)?;
    }

    let result = submit(
        &state,
        "set_trust_signal_schema_version",
        &payload.transaction_base64,
        payload.skip_preflight,
        payload.max_retries,
        payload.timeout_ms,
        payload.poll_ms,
    )
    .await?;

    Ok(Json(LoyaltyTrustActionResponse {
        action: "set_trust_schema_version",
        organizer_id: payload.organizer_id,
        event_id: None,
        wallet: None,
        signal_id: Some(payload.schema_version.to_string()),
        signature: result.0,
        confirmation_status: result.1,
    }))
}

pub async fn get_loyalty(
    auth: AuthContext,
    State(state): State<AppState>,
    Query(query): Query<LoyaltyQuery>,
) -> AppResult<Json<LoyaltyReadResponse>> {
    if let Some(org) = query.organizer_id.as_deref() {
        require_loyalty_reader(&auth, org)?;
    } else {
        require_any_role(
            &auth,
            &[
                Role::ProtocolAdmin,
                Role::OrganizerAdmin,
                Role::Operator,
                Role::Financier,
                Role::Scanner,
            ],
        )?;
    }

    let rows =
        crud::list_loyalty(&state.mongo, &query.wallet, query.organizer_id.as_deref()).await?;
    Ok(Json(LoyaltyReadResponse { rows }))
}

pub async fn get_trust_signals(
    auth: AuthContext,
    State(state): State<AppState>,
    Query(query): Query<TrustSignalQuery>,
) -> AppResult<Json<TrustSignalReadResponse>> {
    if let Some(org) = query.organizer_id.as_deref() {
        require_loyalty_reader(&auth, org)?;
    } else {
        require_any_role(
            &auth,
            &[
                Role::ProtocolAdmin,
                Role::OrganizerAdmin,
                Role::Operator,
                Role::Financier,
                Role::Scanner,
            ],
        )?;
    }

    let rows = crud::list_trust_signals(
        &state.mongo,
        query.wallet.as_deref(),
        query.organizer_id.as_deref(),
        query.event_id.as_deref(),
        query.limit.unwrap_or(100),
    )
    .await?;

    Ok(Json(TrustSignalReadResponse { rows }))
}

async fn submit(
    state: &AppState,
    expected_instruction: &str,
    transaction_base64: &str,
    skip_preflight: bool,
    max_retries: usize,
    timeout_ms: u64,
    poll_ms: u64,
) -> Result<(String, Option<String>), ApiError> {
    let result = state
        .chain_service
        .submit_and_confirm_program_ix(
            SubmitAndConfirmRequest {
                transaction_base64: transaction_base64.to_string(),
                skip_preflight,
                max_retries,
                timeout_ms,
                poll_ms,
            },
            &[expected_instruction],
        )
        .await?;
    Ok((result.signature, result.confirmation_status))
}

fn require_loyalty_writer(auth: &AuthContext, organizer_id: &str) -> Result<(), ApiError> {
    require_any_role(
        auth,
        &[Role::ProtocolAdmin, Role::OrganizerAdmin, Role::Operator],
    )?;
    require_organizer_scope(auth, organizer_id)
}

fn require_signal_writer(auth: &AuthContext, organizer_id: &str) -> Result<(), ApiError> {
    require_any_role(
        auth,
        &[
            Role::ProtocolAdmin,
            Role::OrganizerAdmin,
            Role::Operator,
            Role::Scanner,
        ],
    )?;
    require_organizer_scope(auth, organizer_id)
}

fn require_loyalty_reader(auth: &AuthContext, organizer_id: &str) -> Result<(), ApiError> {
    require_any_role(
        auth,
        &[
            Role::ProtocolAdmin,
            Role::OrganizerAdmin,
            Role::Operator,
            Role::Financier,
            Role::Scanner,
        ],
    )?;
    require_organizer_scope(auth, organizer_id)
}
