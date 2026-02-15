use std::sync::Arc;

use ordinator_contracts::TotalSystemSolution;
use ordinator_orchestrator::Orchestrator;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

pub async fn material_clerk_routes(
    state: Arc<Orchestrator<TotalSystemSolution>>,
) -> OpenApiRouter<Arc<Orchestrator<TotalSystemSolution>>>
{
    OpenApiRouter::new()
        .routes(routes!(crate::handlers::material_handlers::check_material))
        // .route("/{asset}/{supervisor_id}", get(status))
        .with_state(state)

    // TODO: Move this logic into the handler
    // let orchestrator = orchestrator.lock().unwrap();

    // Ok(orchestrator
    //     .handle_supervisor_request(supervisor_request)
    //     .await?)
}
