use std::sync::Arc;

use ordinator_contracts::TotalSystemSolution;
use ordinator_orchestrator::Orchestrator;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

pub async fn supervisor_routes(
    state: Arc<Orchestrator<TotalSystemSolution>>,
) -> OpenApiRouter<Arc<Orchestrator<TotalSystemSolution>>>
{
    OpenApiRouter::new()
        .routes(routes!(crate::handlers::supervisor_handlers::status,))
        .routes(routes!(
            crate::handlers::supervisor_handlers::all_available_technicians
        ))
        // .route("/{asset}/{supervisor_id}", get(status))
        .with_state(state)

    // TODO [ ] Put these into the handler
    // let orchestrator = orchestrator.lock().unwrap();

    // Ok(orchestrator
    //     .handle_supervisor_request(supervisor_request)
    //     .await?)
}
