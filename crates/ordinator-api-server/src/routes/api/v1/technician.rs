use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::AppState;

pub async fn technician_routes(state: AppState) -> OpenApiRouter<AppState>
{
    OpenApiRouter::new()
        .routes(routes!(
            crate::handlers::operational_handlers::activities_for_technician,
        ))
        .routes(routes!(
            crate::handlers::operational_handlers::operational_ids
        ))
        // .route("/{asset}/{supervisor_id}", get(status))
        .with_state(state)

    // TODO [ ] Put these into the handler
    // let orchestrator = orchestrator.lock().unwrap();

    // Ok(orchestrator
    //     .handle_supervisor_request(supervisor_request)
    //     .await?)
}
