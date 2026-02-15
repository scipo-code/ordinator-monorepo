use std::sync::Arc;

use ordinator_contracts::TotalSystemSolution;
use ordinator_orchestrator::Orchestrator;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

// TODO: Replace with updated implementation
pub async fn scheduler_nest(
    state: Arc<Orchestrator<TotalSystemSolution>>,
) -> OpenApiRouter<Arc<Orchestrator<TotalSystemSolution>>>
{
    OpenApiRouter::new()
        .routes(routes!(
            crate::handlers::strategic_handlers::get_scheduler_work_orders
        ))
        .routes(routes!(
            crate::handlers::strategic_handlers::get_single_work_order_with_schedule
        ))
        .routes(routes!(
            crate::handlers::strategic_handlers::work_orders_with_suggested_period,
        ))
        .routes(routes!(
            crate::handlers::strategic_handlers::assign_work_order_to_period,
        ))
        .with_state(state)
}
