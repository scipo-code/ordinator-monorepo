use std::sync::Arc;

use ordinator_contracts::TotalSystemSolution;
use ordinator_orchestrator::Orchestrator;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

pub async fn solution_status_routes(
    state: Arc<Orchestrator<TotalSystemSolution>>,
) -> OpenApiRouter<Arc<Orchestrator<TotalSystemSolution>>>
{
    OpenApiRouter::new()
        .routes(routes!(
            crate::handlers::solution_status_handlers::weekly_solution_status
        ))
        .routes(routes!(
            crate::handlers::solution_status_handlers::project_solution_status
        ))
        .routes(routes!(
            crate::handlers::solution_status_handlers::daily_solution_status
        ))
        // .route("/{asset}/{daily_id}", get(status))
        .with_state(state)

    // TODO: Move the daily request handling into the handler
    // let orchestrator = orchestrator.lock().unwrap();

    // Ok(orchestrator
    //     .handle_daily_request(daily_request)
    //     .await?)
}
