use std::sync::Arc;

use anyhow::Context;
use anyhow::anyhow;
use axum::Json;
use axum::debug_handler;
use axum::extract::Path;
use axum::extract::State;
use axum::response::Result;
use ordinator_contracts::AssetNames;
use ordinator_contracts::TotalSystemSolution;
use ordinator_contracts::supervisor::SupervisorMainTableDto;
use ordinator_contracts::supervisor::SupervisorResourcesDto;
use ordinator_contracts::supervisor::SupervisorResponseMessageDto;
use ordinator_orchestrator::Asset;
use ordinator_orchestrator::Orchestrator;
use ordinator_orchestrator::SupervisorRequestMessage;
use ordinator_orchestrator::SupervisorStatusMessage::General;

use crate::routes::api::AppError;

#[debug_handler]
#[utoipa::path(
    get,
    tag = "Supervisor",
    path = "/{asset}/{supervisor_id}",
    params (
        ("asset" = AssetNames, Path),
        ("supervisor_id" = String, Path),
    ),
    responses(
        (status = 200, body = SupervisorResponseMessageDto),
        (status = 404, body = AppError),
        (status = 500, body = AppError),
    )
)]
pub async fn status(
    State(orchestrator): State<Arc<Orchestrator<TotalSystemSolution>>>,
    Path((asset, supervisor_id)): Path<(Asset, String)>,
) -> Result<Json<SupervisorResponseMessageDto>, AppError>
{
    let lock = orchestrator.actor_registries.lock().unwrap();
    let supervisor_agent_senders = &lock
        .get(&asset)
        .with_context(|| format!("Asset {asset} is not present in the ActorRegistry"))
        .unwrap()
        .supervisor_agent_senders;

    let supervisor_id = supervisor_agent_senders
        .keys()
        .find(|e| e.0 == supervisor_id)
        .ok_or(AppError::Anyhow(
            anyhow!("Supervisor Not found").to_string(),
        ))?;

    let communication = supervisor_agent_senders
        .get(supervisor_id)
        .ok_or(AppError::Anyhow(
            anyhow!("Supervisor not found").to_string(),
        ))?;

    communication
        .from_agent(SupervisorRequestMessage::Status(General))
        .unwrap();

    Ok(Json(communication.from_actor().into()))
}

#[debug_handler]
#[utoipa::path(
    get,
    path = "/all_technicians/{asset}/{supervisor_id}",
    tag = "Supervisor",
    params (
        ("asset" = AssetNames, Path),
        ("supervisor_id" = String, Path),
    ),
    responses(
        (status = 200, body = SupervisorResponseMessageDto),
        (status = 404, body = AppError),
        (status = 500, body = AppError),
    )
)]
pub async fn all_available_technicians(
    State(orchestrator): State<Arc<Orchestrator<TotalSystemSolution>>>,
    // TODO [ ]
    // The `_supervisor_id` should be used in the future when we have additional
    Path((asset, _supervisor_id)): Path<(AssetNames, String)>,
) -> Result<Json<SupervisorResourcesDto>, AppError>
{
    // let lock = orchestrator.actor_registries.lock().unwrap();
    let asset = Asset::try_from(asset).map_err(|e| AppError::Anyhow(e.to_string()))?;

    let supervisor_resources: SupervisorResourcesDto = orchestrator
        .system_solutions
        .lock()
        .expect("SystemSolution locks unavailable")
        .get(&asset)
        .ok_or(anyhow!(
            "SystemSolution for Asset: {} not available",
            &asset
        ))
        .map_err(|e| AppError::Anyhow(e.to_string()))?
        .load()
        .supervisor
        .as_ref()
        .ok_or(AppError::Anyhow(
            "SupervisorSolution not available. Likely due to the Supervisor not being instantiated"
                .to_string(),
        ))?
        .clone()
        .into();

    // ISSUE #000
    // This code should be used for the Command part of the CQRS pattern
    // let supervisor_agent_senders = &lock
    //     .get(&asset.clone())
    //     .with_context(|| format!("Asset {asset:?} is not present in the
    // ActorRegistry"))     .unwrap()
    //     .supervisor_agent_senders;

    // let supervisor_id = supervisor_agent_senders
    //     .keys()
    //     .find(|e| e.0 == supervisor_id)
    //     .ok_or(AppError::Anyhow(
    //         anyhow!("Supervisor Not found").to_string(),
    //     ))?;

    // let communication = supervisor_agent_senders
    //     .get(supervisor_id)
    //     .ok_or(AppError::Anyhow(
    //         anyhow!("Supervisor not found").to_string(),
    //     ))?;

    // communication
    //     .from_agent(SupervisorRequestMessage::Status(General))
    //     .unwrap();

    Ok(Json(supervisor_resources))
}

#[debug_handler]
#[utoipa::path(
    get,
    path = "/supervisor_main_table/{asset}/{supervisor_id}",
    tag = "Supervisor",
    params (
        ("asset" = AssetNames, Path),
        ("supervisor_id" = String, Path),
    ),
    responses(
        (status = 200, body = SupervisorMainTableDto),
        (status = 404, body = AppError),
        (status = 500, body = AppError),
    )
)]
pub async fn supervisor_main_table(
    State(orchestrator): State<Arc<Orchestrator<TotalSystemSolution>>>,
    // TODO [ ]
    // The `_supervisor_id` should be used in the future when we have additional
    Path((asset, _supervisor_id)): Path<(AssetNames, String)>,
) -> Result<Json<SupervisorMainTableDto>, AppError>
{
    let asset = Asset::try_from(asset)
        .map_err(|e| AppError::Anyhow(e.to_string() + "Could not parse the Asset parameter"))?;
    let schedulingenvironment_lock = &orchestrator.scheduling_environment.lock().unwrap();
    let work_orders = &schedulingenvironment_lock.work_orders;
    let time_environment = &schedulingenvironment_lock.time_environment;
    let system_solution = &(**orchestrator
        .system_solutions
        .lock()
        .unwrap()
        .get(&asset)
        .with_context(|| format!("Asset: {asset} is not present in the SystemSolution"))
        .map_err(|e| AppError::Anyhow(e.to_string() + "could not extract the SystemSolution"))?
        .load());

    let supervisor_main_table_dto =
        SupervisorMainTableDto::try_from((work_orders, system_solution, time_environment))
            .with_context(|| {
                format!("SupervisorMainTable could not be constructed for {_supervisor_id}")
            })
            .map_err(|e| {
                AppError::Anyhow(e.to_string() + "could not create the SupervisorMainTableDto")
            })?;
    // let lock = orchestrator.actor_registries.lock().unwrap();

    dbg!(&supervisor_main_table_dto);
    Ok(Json(supervisor_main_table_dto))
}
// _ISSUE_ #000 means unassigned
// TODO [ ] ISSUE #000
// You should craft the needed requests here. You should not be working on the
// Making a general function to handle every type of request to each actor, is
// a good idea. You should make this after the system is up and running.
// pub async fn handle_supervisor_request<Ss>(
//     State(orchestrator): State<Arc<Mutex<Orchestrator<Ss>>>>,
//     supervisor_request: SupervisorRequest,
// ) -> Result<HttpResponse, actix_web::Error>
// where
//     Ss: SystemSolutionTrait,
// {
//     event!(Level::INFO, supervisor_request = ?supervisor_request);
//     let supervisor_agent_addrs = match
// self.agent_registries.get(&supervisor_request.asset) {
//         Some(agent_registry) => &agent_registry.supervisor_agent_senders,
//         None => {
//             return Ok(HttpResponse::BadRequest()
//                 .json("SUPERVISOR: SUPERVISOR AGENT NOT INITIALIZED FOR THE
// ASSET"));         }
//     };
//     let supervisor_agent_addr = supervisor_agent_addrs
//                 .iter()
//                 .find(|(id, _)| id.0 ==
// supervisor_request.supervisor.to_string())                 .expect("This will
// error at somepoint you will need to handle if you have added additional
// supervisors")                 .1;

//     // This was the reason that we wanted the tokio runtime.
//     supervisor_agent_addr
//         .sender
//         .send(crate::agents::ActorMessage::Actor(
//             supervisor_request.supervisor_request_message,
//         ))
//         .map_err(actix_web::error::ErrorInternalServerError)?;

//     let response = supervisor_agent_addr
//         .receiver
//         .recv()
//         .map_err(actix_web::error::ErrorInternalServerError)?
//         .map_err(actix_web::error::ErrorInternalServerError)?;

//     let supervisor_response =
// SupervisorResponse::new(supervisor_request.asset, response);

//     let system_responses = SystemResponses::Supervisor(supervisor_response);
//     Ok(HttpResponse::Ok().json(system_responses))
// }
