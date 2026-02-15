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
            crate::handlers::supervisor_handlers::all_technicians
        ))
        .routes(routes!(
            crate::handlers::supervisor_handlers::technician_availability
        ))
        .routes(routes!(
            crate::handlers::supervisor_handlers::assign_to_technicians
        ))
        .routes(routes!(
            crate::handlers::supervisor_handlers::supervisor_main_table
        ))
        .routes(routes!(
            crate::handlers::supervisor_handlers::add_technician
        ))
        .with_state(state)
}
