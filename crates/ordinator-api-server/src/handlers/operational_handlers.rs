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

// Handlers create messages directly rather than using enums for routes to avoid tight coupling
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
    Path(asset): Path<AssetNames>,
) -> Result<Json<Vec<IdDto>>, AppError>
{
    let asset = Asset::try_from(asset).map_err(|e| AppError::Anyhow(e.to_string()))?;

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
            .find(|actor_solution| actor_solution.0.0 == technician_id)
            .ok_or(AppError::Anyhow(format!("Technician: {technician_id} is not part of the scheduling system\n\nEither you:\n\twrote the ID wrong\n\tThe person has not been added to the system")))?
            .1
            .all_scheduled_work_order_activities()
            .iter()
            .map(|e|e.clone().into())
            .collect::<Vec<_>>()
    ))
}

#[allow(unused)]
pub async fn operational_handler_for_operational_agent(
    State(orchestrator): State<Arc<Orchestrator<TotalSystemSolution>>>,
    Path(asset): Path<AssetNames>,
    Path(technician_id): Path<String>,
) -> Result<Json<OperationalResponseMessage>>
{
    let asset = Asset::try_from(asset).map_err(|e| AppError::Anyhow(e.to_string()))?;

    // Create operational status request
    let operational_request_message =
        OperationalRequestMessage::Status(OperationalStatusRequest::General);
    let hash_map = orchestrator.actor_registries.lock().unwrap();
    let communication = hash_map
        .get(&asset)
        .expect("This error should be handled higher up")
        .get_operational_addr(&technician_id)
        .context("OperationalCommunication not found")?;

    communication
        .from_agent(operational_request_message)
        .context("Could not await the message sending, thread problems are the most likely")?;

    let response = communication.from_actor();

    Ok(Json(response))
}
