use std::sync::Arc;

use anyhow::Context;
use anyhow::Result;
use axum::Json;
use axum::debug_handler;
use axum::extract::Path;
use axum::extract::State;
use ordinator_contracts::AssetNames;
use ordinator_contracts::IdDto;
use ordinator_contracts::TotalSystemSolution;
use ordinator_contracts::technician::OperationalAssignmentsDto;
use ordinator_orchestrator::Asset;
use ordinator_orchestrator::OperationalRequestMessage;
use ordinator_orchestrator::OperationalResponseMessage;
use ordinator_orchestrator::OperationalStatusRequest;
use ordinator_orchestrator::Orchestrator;

use crate::routes::api::AppError;

// CRUCIAL INSIGHT: Making enums for handlers and routes is a horrible idea. It
// becomes difficult to change things and everything becomes coupled.
//
// The handlers should create the Messages. That means that the
// is this even worth it? I am not really sure. I think that the
// best approach is to create something that will allow us... Just
// do it! Work fast and be prepared to change things back.
#[debug_handler]
#[utoipa::path(
    get,
    path = "/all_technicians/{asset}",
    tag = "Technician",
    params (
        ("asset" = AssetNames, Path),
    ),
    responses(
        (status = 200, body = Vec<IdDto>),
        (status = 404, body = AppError),
        (status = 500, body = AppError),
    )
)]
#[allow(unused)]
pub async fn operational_ids(
    State(orchestrator): State<Arc<Orchestrator<TotalSystemSolution>>>,
    // This is actually not the best way of coding it?
    Path(asset): Path<Asset>,
) -> Result<Json<Vec<IdDto>>, AppError>
{
    Ok(Json(
        orchestrator
            .actor_registries
            .lock()
            .unwrap()
            .get(&asset)
            .expect("This error should be handled higher up")
            .operational_agent_senders
            .keys()
            .map(|e| IdDto::from(e.clone()))
            .collect(),
    ))
}

#[debug_handler]
#[utoipa::path(
    get,
    path = "/technician/assigned_activities/{asset}/{id}",
    tag = "Technician",
    params (
        ("asset" = AssetNames, Path),
        ("id" = String, Path),
    ),
    responses(
        (status = 200, body = Vec<IdDto>),
        (status = 404, body = AppError),
        (status = 500, body = AppError),
    )
)]
pub async fn activities_for_technician(
    State(orchestrator): State<Arc<Orchestrator<TotalSystemSolution>>>,
    Path((asset, technician_id)): Path<(Asset, String)>,
) -> Result<Json<Vec<OperationalAssignmentsDto>>, AppError>
{
    Ok(Json(
        orchestrator
            .system_solutions
            .lock()
            .expect("Mutex for SystemSolution for a specific Asset should never Error")
            .get(&asset).ok_or(AppError::Anyhow(format!("SystemSolution for Asset: {} not found", asset.clone())))?
            .load()
            .operational
            .iter()
            .find(|e| e.0.0 == technician_id)
            .ok_or(AppError::Anyhow(format!("Technician: {technician_id} is not part of the scheduling system\n\nEither you:\n\twrote the ID wrong\n\tThe person has not been added to the system")))?
            .1
            .scheduled_work_order_activities
            .iter()
            .map(|e|e.clone().into())
            .collect::<Vec<_>>()
    ))
}

#[allow(unused)]
pub async fn operational_handler_for_operational_agent(
    State(orchestrator): State<Arc<Orchestrator<TotalSystemSolution>>>,
    Path(asset): Path<Asset>,
    Path(technician_id): Path<String>,
) -> Result<Json<OperationalResponseMessage>>
{
    // INFO; So state link should not be possible to send here. This will be
    // a very good exercise in how to use the `pub` keyword in a good way.
    // I really think that a large `enum` is a fine approach. Otherwise
    // we will have to turn the many types into a
    // Should
    let operational_request_message =
        OperationalRequestMessage::Status(OperationalStatusRequest::General);
    // OperationalStatusMessage
    let hash_map = orchestrator.actor_registries.lock().unwrap();
    let communication = hash_map
        .get(&asset)
        .expect("This error should be handled higher up")
        .get_operational_addr(&technician_id)
        .context("OperationalCommunication not found")?;

    communication
        .from_agent(operational_request_message)
        .context("Could not await the message sending, theard problems are the most likely")?;

    let response = communication.from_actor();

    Ok(Json(response))
}

// pub async fn operational_handler_alloperationalstatus(Path(asset):
// Path<Asset>) -> Result<Json> {

// }
//             let operational_request_status =
// OperationalStatusRequest::General;             let
// operational_request_message =     pub async fn
// operational_handler_operationalrequestmessage() -> {

//     }
//
// OperationalRequestMessage::Status(operational_request_status);
// let mut operational_responses: Vec<OperationalResponseMessage> = vec![];

//             let agent_registry_option = self.agent_registries.get(&asset);

//             let agent_registry = match agent_registry_option {
//                 Some(agent_registry) => agent_registry,
//                 None => {
//                     return Ok(HttpResponse::BadRequest()
//                         .json("STRATEGIC: STRATEGIC AGENT NOT INITIALIZED FOR
// THE ASSET"));                 }
//             };

//             for operational_addr in
// agent_registry.operational_agent_senders.values() {
// operational_addr                     .sender
//                     .send(crate::agents::ActorMessage::Actor(
//                         operational_request_message.clone(),
//                     ))
//                     .unwrap();
//             }

//             for operational_addr in
// agent_registry.operational_agent_senders.values() {                 let
// response = operational_addr.receiver.recv().unwrap().unwrap();

//                 operational_responses.push(response);
//             }
//             OperationalResponse::AllOperationalStatus(operational_responses)
//         }
//     };
//     let system_responses =
// SystemResponses::Operational(operational_response);
//     Ok(HttpResponse::Ok().json(system_responses))
// }
