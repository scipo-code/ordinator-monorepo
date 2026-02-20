use std::sync::Arc;

use ordinator_contracts::TotalSystemSolution;
use ordinator_orchestrator::Orchestrator;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

pub async fn daily_routes(
    state: Arc<Orchestrator<TotalSystemSolution>>,
) -> OpenApiRouter<Arc<Orchestrator<TotalSystemSolution>>>
{
    OpenApiRouter::new()
        .routes(routes!(crate::handlers::daily_handlers::status,))
        .routes(routes!(
            crate::handlers::daily_handlers::all_technicians
        ))
        .routes(routes!(
            crate::handlers::daily_handlers::technician_availability
        ))
        .routes(routes!(
            crate::handlers::daily_handlers::assign_to_technicians
        ))
        .routes(routes!(
            crate::handlers::daily_handlers::daily_main_table
        ))
        .routes(routes!(
            crate::handlers::daily_handlers::add_technician
        ))
        .with_state(state)
}
