use std::sync::Arc;

use anyhow::Context;
use axum::Json;
use axum::extract::Path;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::response::Result;
use chrono::NaiveDate;
use chrono::ParseError;
use ordinator_contracts::AssetNames;
use ordinator_contracts::NaiveDateDto;
use ordinator_contracts::WorkOrderNumberDto;
use ordinator_orchestrator::Asset;
use ordinator_orchestrator::Orchestrator;
use ordinator_orchestrator::StateLink;
use ordinator_orchestrator::SystemSolutions;
use ordinator_orchestrator::TacticalRequestMessage;
use ordinator_orchestrator::TacticalStatusMessage;

// So each handler should construct a specific message. That is the key point
// here. This function uses the orchestrator to send any kind of message. Which
// way is the correct one here?
//
// ESSAY: What is the best thing to put into the `TacticalRequest`? The
// fundamental issue here is what should be in the URL. I think that we should
// put the data inside of the messages into a JSON but that the handlers should
// only take a single RequestMessage and a corresponding `<Actor>StatusMessage`.
// That means that the handler here should only construct a single message
#[utoipa::path(
    get,
    tag = "Scheduler",
    path = "/tactical_algorithm_status",
    responses((status = 200, body = [Vec<String>]))
)]
pub async fn status<Ss>(
    State(orchestrator): State<Arc<Orchestrator<Ss>>>,
    Path(asset): Path<Asset>,
) -> Result<Response>
where
    Ss: SystemSolutions,
{
    let message = TacticalRequestMessage::Status(TacticalStatusMessage::General);

    let hash_map = orchestrator.actor_registries.lock().unwrap();
    let actor_registry_for_asset = &hash_map
        .get(&asset)
        .unwrap()
        // .with_context(|| format!("Asset {} not initialized", &asset))?
        .tactical_agent_sender;

    actor_registry_for_asset.from_agent(message).unwrap();

    let response = actor_registry_for_asset
        .receiver_from_actor
        .recv()
        .unwrap()
        .unwrap();

    Ok(Json(response).into_response())
}

// #[debug_handler]
// #[utoipa::path(
//     get,
//     tag = "Supervisor",
//     path = "/{asset}/{supervisor_id}",
//     params (
//         ("asset" = AssetNames, Path),
//         ("supervisor_id" = String, Path),
//     ),
//     responses(
//         (status = 200, body = SupervisorResponseMessageDto),
//         (status = 404, body = AppError),
//         (status = 500, body = AppError),
//     )
// )]
#[utoipa::path(
    get,
    tag = "Scheduler",
    path = "/start_days_for_activities/{asset}",
    params (
        ("asset" = AssetNames, Path),
    ),
    responses((status = 200, body = [Vec<String>]))
)]
pub async fn start_days_for_activities<Ss>(
    State(orchestrator): State<Arc<Orchestrator<Ss>>>,
    Path(asset): Path<Asset>,
) -> Result<Response, AppError>
where
    Ss: SystemSolutions,
{
    let tactical_days = orchestrator
        .system_solutions
        .lock()
        .unwrap_or_else(|_| panic!("Could not lock the SystemSolution for Asset: {}", &asset))
        .get(&asset)
        .with_context(|| format!("SystemSolution for Asset: {} does not exist", &asset))
        .map_err(|e| AppError::Anyhow(e.to_string()))?
        .load()
        .tactical_actor_solution()
        .map_err(|_| AppError::Anyhow(format!("No TacticalSolution exists for Asset: {}", &asset)))?
        .all_scheduled_tasks();

    Ok(Json(tactical_days).into_response())
}

#[utoipa::path(
    patch,
    tag = "Scheduler",
    path = "/assign_start_day_for_work_order/{asset}/{work_order_number}/",
    params (
        ("asset" = AssetNames, Path),
        ("work_order_number" = WorkOrderNumberDto, Path),
    ),
    request_body(content = NaiveDateDto, description = "json with a basic start date", content_type = "application/json", example = json!("2025-02-25")),
    responses((status = 200, body = [Vec<String>]))
)]
pub async fn assign_start_day_for_work_order<Ss>(
    State(orchestrator): State<Arc<Orchestrator<Ss>>>,
    // TODO [ ] `asset` should be used for authentication.
    Path((_asset, work_order_number)): Path<(Asset, WorkOrderNumber)>,
    Json(basic_start_date_dto): Json<NaiveDateDto>,
) -> Result<Response, AppError>
where
    Ss: SystemSolutions,
{
    let basic_start_date: NaiveDate = basic_start_date_dto
        .try_into()
        .map_err(|e: ParseError| AppError::Anyhow(e.to_string()))?;
    let mut scheduling_environment_lock = orchestrator.scheduling_environment.lock().unwrap();

    let asset = Asset::try_from(_asset).map_err(|e| AppError::Anyhow(e.to_string()))?;

    let materials_to_periods = &scheduling_environment_lock
        .worker_environment
        .actor_specification
        .get(&asset)
        .context("Asset not available in ActorSpecifications")
        .map_err(|e| AppError::Anyhow(e.to_string()))?
        .material_to_period
        .clone();

    let days = &scheduling_environment_lock.time_environment.days.clone();
    let periods = &scheduling_environment_lock.time_environment.periods.clone();
    let day = days
        .iter()
        .find(|day| day.date == basic_start_date)
        .with_context(|| "Chosen start date is outside of the valid scheduling period".to_string())
        .map_err(|e| AppError::Anyhow(e.to_string()))?;

    scheduling_environment_lock
        .work_orders
        .inner
        .get_mut(&work_order_number)
        .with_context(|| format!("{work_order_number:#?} not found in SchedulingEnvironment"))
        .map_err(|e| AppError::Anyhow(e.to_string()))?
        // TODO [ ] You should clearly differentiate between the WO modifying code and the
        // normal code.
        .set_basic_start_date(basic_start_date);

    drop(scheduling_environment_lock);
    let mut scheduling_environment_lock = orchestrator.scheduling_environment.lock().unwrap();
    let work_order = scheduling_environment_lock
        .work_orders
        .inner
        .get(&work_order_number)
        .with_context(|| format!("WorkOrder {work_order_number:#?} is not in WorkOrders"))
        .map_err(|e| AppError::Anyhow(e.to_string()))?
        .forced_work_order(periods, days, materials_to_periods)
        .map_err(|e| AppError::Anyhow(e.to_string()))?;

    scheduling_environment_lock
        .assignments
        .make_assignment_for_tactical(work_order_number, &work_order, day.clone())
        .with_context(|| "Could not make a tactical assignment".to_string())
        .map_err(|e| AppError::Anyhow(e.to_string()))?;

    orchestrator
        .state_link_bus
        .lock()
        .unwrap()
        .broadcast(StateLink::WorkOrders(vec![work_order_number]));
    Ok(format!(
        "Command successfully processed in the system\nWork order {work_order_number} was correctly set to basic start date {basic_start_date}"
    ).into_response())
}

#[utoipa::path(
    get,
    tag = "Scheduler",
    path = "/daily_loadings/{asset}",
    params (
        ("asset" = AssetNames, Path),
    ),
    responses((status = 200, body = [Vec<String>]))
)]
pub async fn daily_loadings<Ss>(
    State(orchestrator): State<Arc<Orchestrator<Ss>>>,
    Path(asset): Path<Asset>,
) -> Result<Response, AppError>
where
    Ss: SystemSolutions,
{
    let tactical_days = orchestrator
        .system_solutions
        .lock()
        .unwrap_or_else(|_| panic!("Could not lock the SystemSolution for Asset: {}", &asset))
        .get(&asset)
        .with_context(|| format!("SystemSolution for Asset: {} does not exist", &asset))
        .map_err(|e| AppError::Anyhow(e.to_string()))?
        .load()
        .tactical_actor_solution()
        .map_err(|_| AppError::Anyhow(format!("No TacticalSolution exists for Asset: {}", &asset)))?
        .tactical_loadings();

    Ok(Json(tactical_days).into_response())
}

use ordinator_orchestrator::TacticalInterface;
use ordinator_orchestrator::WorkOrderNumber;

use crate::routes::api::AppError;
