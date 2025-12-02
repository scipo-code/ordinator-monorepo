use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::AppState;

pub async fn supervisor_routes(state: AppState) -> OpenApiRouter<AppState>
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
        // .route("/{asset}/{supervisor_id}", get(status))
        .with_state(state)

    // TODO [ ] Put these into the handler
    // let orchestrator = orchestrator.lock().unwrap();

    // Ok(orchestrator
    //     .handle_supervisor_request(supervisor_request)
    //     .await?)
}
