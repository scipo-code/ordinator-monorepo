use std::sync::Arc;

use axum::routing::get;
use ordinator_contracts::TotalSystemSolution;
use ordinator_orchestrator::Orchestrator;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::AppState;
use crate::handlers::tactical_handlers::status;

// Making a `status` for each actor is probably a really good idea.
pub async fn tactical_route(state: AppState) -> OpenApiRouter<AppState>
{
    OpenApiRouter::new()
        .route("/", get(status))
        .routes(routes!(
            crate::handlers::tactical_handlers::start_days_for_activities,
        ))
        .routes(routes!(crate::handlers::tactical_handlers::daily_loadings,))
        .routes(routes!(
            crate::handlers::tactical_handlers::assign_start_day_for_work_order
        ))
        .with_state(state)
}

// TODO [ ]
// let orchestrator = orchestrator.lock().unwrap();
// Ok(orchestrator
//     .handle_tactical_request(tactical_request)
//     .await?)
//
// NOTE [ ] 2025-07-02 here is how you do.
// .routes(routes!(
//     crate::handlers::supervisor_handlers::all_available_technicians
// ))
