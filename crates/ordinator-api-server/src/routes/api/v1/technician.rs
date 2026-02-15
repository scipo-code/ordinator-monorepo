use std::sync::Arc;

use ordinator_contracts::TotalSystemSolution;
use ordinator_orchestrator::Orchestrator;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

pub async fn technician_routes(
    state: Arc<Orchestrator<TotalSystemSolution>>,
) -> OpenApiRouter<Arc<Orchestrator<TotalSystemSolution>>>
{
    OpenApiRouter::new()
        .routes(routes!(
            crate::handlers::operational_handlers::activities_for_technician,
        ))
        .routes(routes!(
            crate::handlers::operational_handlers::operational_ids
        ))
        .with_state(state)

    // TODO: Move these into the handler
    // let orchestrator = orchestrator.lock().unwrap();

    // Ok(orchestrator
    //     .handle_supervisor_request(supervisor_request)
    //     .await?)
}
