use axum::{Json, extract::State};

use crate::{
    app::AppState,
    error::{ApiError, AppResult},
    module::{
        auth::{
            guard::{AuthContext, require_any_role, require_organizer_scope},
            model::Role,
        },
        resale_compiler::schema::{ResaleSimulationRequest, ResaleSimulationResponse},
    },
};

pub async fn simulate_resale_policy(
    auth: AuthContext,
    State(state): State<AppState>,
    Json(payload): Json<ResaleSimulationRequest>,
) -> AppResult<Json<ResaleSimulationResponse>> {
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

    let Some(service) = &state.resale_compiler_service else {
        return Err(ApiError::DatabaseUnavailable);
    };

    let response = service.simulate(payload).await?;
    Ok(Json(response))
}
