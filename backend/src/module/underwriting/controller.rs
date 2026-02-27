use axum::{Json, extract::State};

use crate::{
    app::AppState,
    error::{ApiError, AppResult},
    module::{
        auth::{
            guard::{AuthContext, require_any_role, require_organizer_scope},
            model::Role,
        },
        underwriting::schema::{UnderwritingRequest, UnderwritingResponse},
    },
};

pub async fn evaluate_underwriting(
    auth: AuthContext,
    State(state): State<AppState>,
    Json(payload): Json<UnderwritingRequest>,
) -> AppResult<Json<UnderwritingResponse>> {
    require_any_role(
        &auth,
        &[
            Role::ProtocolAdmin,
            Role::OrganizerAdmin,
            Role::Operator,
            Role::Financier,
        ],
    )?;
    require_organizer_scope(&auth, &payload.organizer_id)?;

    let Some(service) = &state.underwriting_service else {
        return Err(ApiError::DatabaseUnavailable);
    };

    let decision = service.evaluate(payload).await?;
    Ok(Json(UnderwritingResponse { decision }))
}
