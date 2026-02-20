use std::sync::Arc;

use axum::routing::get;
use ordinator_contracts::TotalSystemSolution;
use ordinator_orchestrator::Orchestrator;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::handlers::project_handlers::status;

// Create status endpoint for each actor in the orchestrator
pub async fn project_route(
    state: Arc<Orchestrator<TotalSystemSolution>>,
) -> OpenApiRouter<Arc<Orchestrator<TotalSystemSolution>>>
{
    OpenApiRouter::new()
        .route("/", get(status::<TotalSystemSolution>))
        .routes(routes!(
            crate::handlers::project_handlers::start_days_for_activities,
        ))
        .routes(routes!(crate::handlers::project_handlers::daily_loadings,))
        .routes(routes!(
            crate::handlers::project_handlers::assign_start_day_for_work_order
        ))
        .with_state(state)
}

// TODO: Implement orchestrator request handling
// let orchestrator = orchestrator.lock().unwrap();
// Ok(orchestrator
//     .handle_project_request(project_request)
//     .await?)
//
// TODO: Add daily handlers for available technicians
// .routes(routes!(
//     crate::handlers::daily_handlers::all_available_technicians
// ))
