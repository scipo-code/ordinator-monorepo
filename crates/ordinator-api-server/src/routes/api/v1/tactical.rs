use std::sync::Arc;

use axum::routing::get;
use ordinator_contracts::TotalSystemSolution;
use ordinator_orchestrator::Orchestrator;
use utoipa_axum::router::OpenApiRouter;

use crate::handlers::tactical_handlers::status;

// Making a `status` for each actor is probably a really good idea.
pub async fn tactical_route(
    state: Arc<Orchestrator<TotalSystemSolution>>,
) -> OpenApiRouter<Arc<Orchestrator<TotalSystemSolution>>>
{
    OpenApiRouter::new()
        .route("/", get(status::<TotalSystemSolution>))
        .with_state(state)
}

// TODO [ ]
// let orchestrator = orchestrator.lock().unwrap();
// Ok(orchestrator
//     .handle_tactical_request(tactical_request)
//     .await?)
