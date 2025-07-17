use std::sync::Arc;

use ordinator_contracts::TotalSystemSolution;
use ordinator_orchestrator::Orchestrator;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

// TODO [x]
// The main idea is to replace all the.
pub async fn scheduler_nest(
    state: Arc<Orchestrator<TotalSystemSolution>>,
) -> OpenApiRouter<Arc<Orchestrator<TotalSystemSolution>>>
{
    OpenApiRouter::new()
        .routes(routes!(
            crate::handlers::strategic_handlers::get_scheduler_work_orders
        ))
        .routes(routes!(
            crate::handlers::strategic_handlers::period_for_work_order,
        ))
        .routes(routes!(
            crate::handlers::strategic_handlers::assign_work_order_to_period,
        ))
        .with_state(state)
}
// let asset = strategic_request.asset;
// let orchestrator_guard = orchestrator.lock().unwrap();

// let strategic = &orchestrator_guard
//     .agent_registries
//     .get(&asset)
//     .unwrap()
//     .strategic_agent_sender;

// strategic
//     .sender
//     .send(crate::agents::ActorMessage::Actor(
//         strategic_request.strategic_request_message,
//     ))
//     .map_err(actix_web::error::ErrorInternalServerError)?;

// let response = strategic
//     .receiver
//     .recv()
//     .map_err(actix_web::error::ErrorInternalServerError)?;
// drop(orchestrator_guard);

// let strategic_response_message = match response {
//     Ok(message) => message,
//     Err(e) => {
//         let error = format!("{:?}", e.context("http request could not be
// completed"));         return Ok(HttpResponse::BadRequest().body(error));
//     }
// };

// let strategic_response = StrategicResponse::new(asset,
// strategic_response_message);

// let system_message = SystemResponses::Strategic(strategic_response);

// Ok(HttpResponse::Ok().json(system_message))
