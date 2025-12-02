use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::AppState;

pub async fn material_clerk_routes(state: AppState) -> OpenApiRouter<AppState>
{
    OpenApiRouter::new()
        .routes(routes!(crate::handlers::material_handlers::check_material))
        // .route("/{asset}/{supervisor_id}", get(status))
        .with_state(state)

    // TODO [ ] Put these into the handler
    // let orchestrator = orchestrator.lock().unwrap();

    // Ok(orchestrator
    //     .handle_supervisor_request(supervisor_request)
    //     .await?)
}
