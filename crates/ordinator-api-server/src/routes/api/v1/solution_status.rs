use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::AppState;

pub async fn solution_status_routes(state: AppState) -> OpenApiRouter<AppState>
{
    OpenApiRouter::new()
        .routes(routes!(
            crate::handlers::solution_status_handlers::strategic_solution_status
        ))
        .routes(routes!(
            crate::handlers::solution_status_handlers::tactical_solution_status
        ))
        .routes(routes!(
            crate::handlers::solution_status_handlers::supervisor_solution_status
        ))
        // .route("/{asset}/{supervisor_id}", get(status))
        .with_state(state)

    // TODO [ ] Put these into the handler
    // let orchestrator = orchestrator.lock().unwrap();

    // Ok(orchestrator
    //     .handle_supervisor_request(supervisor_request)
    //     .await?)
}
