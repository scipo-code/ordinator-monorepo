use std::sync::Arc;

use anyhow::Context;
use axum::Json;
use axum::debug_handler;
use axum::extract::Path;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::response::Result;
// use axum::extract::Query;
use axum_extra::extract::Query;
use ordinator_contracts::AssetNames;
use ordinator_contracts::PeriodDto;
use ordinator_contracts::TotalSystemSolution;
use ordinator_contracts::WorkOrderNumberDto;
use ordinator_contracts::scheduler::SchedulerWorkOrderDto;
use ordinator_contracts::scheduler::WorkOrderInfoWithSchedulingDto;
use ordinator_contracts::scheduler::WorkOrderSingleRowSimpleDto;
use ordinator_orchestrator::Asset;
use ordinator_orchestrator::Orchestrator;
use ordinator_orchestrator::StateLink;
use ordinator_orchestrator::StrategicInterface;
use ordinator_orchestrator::SystemSolutions;
use ordinator_orchestrator::WorkOrderNumber;
use serde::Deserialize;
use tracing::instrument;
use utoipa::IntoParams;
use utoipa::ToSchema;

use crate::routes::api::AppError;

// This is a handler. Not a `Route` you should change that. Keep working.
#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct WorkOrdersWithSchedulingQueryParams
{
    #[serde(default)]
    periods: Vec<PeriodDto>,
}

#[debug_handler]
#[utoipa::path(
    get,
    path = "/work_orders_with_scheduling/{asset}",

    tag = "Scheduler",
    params (
        ("asset" = AssetNames, Path),
        WorkOrdersWithSchedulingQueryParams,
    ),
    responses(
        (status = 200, body = WorkOrderSingleRowSimpleDto),
        (status = 404, body = AppError),
        (status = 500, body = AppError),
    )
)]
#[instrument(target = "business_events", fields(elapsed_time), skip(orchestrator))]
pub async fn get_scheduler_work_orders(
    State(orchestrator): State<Arc<Orchestrator<TotalSystemSolution>>>,
    Path(asset): Path<AssetNames>,
    Query(query): Query<WorkOrdersWithSchedulingQueryParams>,
) -> Result<Json<SchedulerWorkOrderDto>, AppError>
{
    let asset = Asset::try_from(asset).map_err(|e| AppError::Anyhow(e.to_string()))?;

    let system_solution = orchestrator
        .system_solutions
        .lock()
        .unwrap()
        .get(&asset.clone())
        .with_context(|| format!("Asset {:?} is not present in the ActorRegistry", &asset))
        .map_err(|e| AppError::Anyhow(e.to_string()))?
        .load();

    let scheduling_environment = orchestrator.scheduling_environment.lock().unwrap();

    let work_order_schedule =
        SchedulerWorkOrderDto::try_from((asset.clone(), scheduling_environment, system_solution))
            .expect("This should never fail");

    let periods = query.periods;
    let work_order_schedule = if !periods.is_empty() {
        let work_order_schedule: Vec<_> = work_order_schedule
            .0
            .into_iter()
            .filter(|wo| periods.contains(&PeriodDto(wo.get_suggested_period())))
            .collect();
        SchedulerWorkOrderDto(work_order_schedule)
    } else {
        work_order_schedule
    };

    Ok(Json(work_order_schedule))
}

#[debug_handler]
#[utoipa::path(
    get,
    path = "/work_orders_with_scheduling/{asset}/{work_order}",

    tag = "Scheduler",
    params (
        ("asset" = AssetNames, Path),
        ("work_order" = WorkOrderNumberDto, Path),
    ),
    responses(
        (status = 200, body = WorkOrderSingleRowSimpleDto),
        (status = 404, body = AppError),
        (status = 500, body = AppError),
    )
)]
pub async fn get_single_work_order_with_schedule(
    State(orchestrator): State<Arc<Orchestrator<TotalSystemSolution>>>,
    Path((asset, work_order)): Path<(AssetNames, WorkOrderNumberDto)>,
) -> Result<Json<WorkOrderInfoWithSchedulingDto>, AppError>
{
    let asset = Asset::try_from(asset).map_err(|e| AppError::Anyhow(e.to_string()))?;

    let system_solution = orchestrator
        .system_solutions
        .lock()
        .unwrap()
        .get(&asset.clone())
        .with_context(|| format!("Asset {:?} is not present in the ActorRegistry", &asset))
        .map_err(|e| AppError::Anyhow(e.to_string()))?
        .load();

    let scheduling_environment = orchestrator.scheduling_environment.lock().unwrap();

    let work_order_payload = WorkOrderInfoWithSchedulingDto::try_from((
        asset.clone(),
        scheduling_environment,
        system_solution,
        work_order,
    ))
    .map_err(|e| AppError::Anyhow(e.to_string()))?;

    Ok(Json(work_order_payload))
}

#[utoipa::path(
    get,
    tag = "Scheduler",
    path = "/work_orders_with_suggested_period/{asset}",
    params (
        ("asset" = AssetNames, Path),
    ),
    responses((status = 200, body = [HashMap<WorkOrderNumberDto, PeriodDto>]))
)]
pub async fn work_orders_with_suggested_period<Ss>(
    State(orchestrator): State<Arc<Orchestrator<Ss>>>,
    Path(asset): Path<Asset>,
) -> Result<Response, AppError>
where
    Ss: SystemSolutions,
{
    let strategic_periods = orchestrator
        .system_solutions
        .lock()
        .unwrap_or_else(|_| panic!("Could not lock the SystemSolution for Asset: {}", &asset));
    let strategic_periods = strategic_periods
        .get(&asset)
        .with_context(|| format!("SystemSolution for Asset: {} does not exist", &asset))
        .map_err(|e| AppError::Anyhow(e.to_string()))?
        .load();
    let strategic_periods = strategic_periods
        .strategic()
        .map_err(|_| {
            AppError::Anyhow(format!("No StrategicSolution exists for Asset: {}", &asset))
        })?
        .all_scheduled_tasks();

    let strategic_periods: Vec<_> = strategic_periods
        .iter()
        .map(|f| (WorkOrderNumberDto(f.0.0), PeriodDto(f.1.period_string())))
        .collect();

    Ok(Json(strategic_periods).into_response())
}

// pub async fn scheduling_status_for_workorder<Ss>(
//     State(orchestrator): State<Arc<Orchestrator<Ss>>>,
//     Path(work_order_number_dto): Path<WorkOrderNumberDto>,
// ) where
//     Ss: SystemSolutions,
// {
// }

/// This handler updates the [`SchedulingEnvironment`] that the UnloadingPoint
/// has been updated and sends a command
/// to every `Actor` that the [`SchedulignEnvironment`] has changed.
#[debug_handler]
#[utoipa::path(
    post,
    path = "/{asset}/assign_work_order_to_period/{work_order_number}",
    tag = "Scheduler",
    params (
        ("asset" = AssetNames, Path),
        ("work_order_number" = WorkOrderNumberDto, Path),
    ),
    request_body(content = PeriodDto, description = "json with the assigned period", content_type = "application/json", example = json!("2025-W5-6")),
    responses(
        (status = 200),
        (status = 404, body = AppError),
        (status = 500, body = AppError),
    )
)]
pub async fn assign_work_order_to_period(
    State(orchestrator): State<Arc<Orchestrator<TotalSystemSolution>>>,
    Path((asset_dto, work_order_number_dto)): Path<(AssetNames, WorkOrderNumberDto)>,
    Json(period_dto): Json<PeriodDto>,
) -> Result<Response, AppError>
{
    // This should go into the handler, directory. There is no other way around it
    // REMEMBER: You should only wrap method calls that the Orchestrator exposes.
    //
    // WARN: You are beginning to feel drained again. You should grap something to
    // eat again.
    // TODO [ ] add a `bus` for each `Asset`
    let _asset = Asset::try_from(asset_dto).map_err(|e| AppError::Anyhow(e.to_string()))?;

    let mut scheduling_environment = orchestrator.scheduling_environment.lock().unwrap();

    let work_order_number = WorkOrderNumber::from(work_order_number_dto);
    let periods = &scheduling_environment.time_environment.periods.clone();

    let period_option = periods
        .iter()
        .find(|per| per.period_string() == period_dto.0)
        .cloned();

    let period = match period_option {
        Some(period) => period,
        None => {
            return Err(AppError::Anyhow(format!(
                "Period: {period_dto:?}\nwas not found in the scheduling_environment\nDid you:\n1. use the pattern yyyy-Wxx-Wzz?\n2. Use the correct even or uneven week numbers correctly"
            )));
        }
    };

    scheduling_environment
        .work_orders
        .update_scheduled_period(&work_order_number, &period, periods)
        .with_context(|| {
            format!(
                "Could assign work order {:#?} to period {}",
                work_order_number,
                period.period_string()
            )
        })
        .map_err(|e| AppError::Anyhow(e.to_string()))?;

    // I hated that message structure when I first wrote it, now you have a lesson
    // to learn. So the [`StateLink`] is responsible for updating the `Actors` when
    // there has been a change to the scheduling_environment. The messages that
    // simply guide the search procedure should be handled the same way as
    // always.
    let state_link = StateLink::WorkOrders(vec![work_order_number]);

    // You are doing what you should have done a long time ago.
    orchestrator
        .state_link_bus
        .lock()
        .unwrap()
        .broadcast(state_link);
    // Send a `StateLink` message to all Actors
    // TODO [x] 2025-07-14 make a broadcast channel.
    // It is very important that you get this right. You should make sure to use the
    // correct construct. I think
    //
    // This is an issue, I think that the best approach here is to make the system
    // work in a way that will not let the system.
    Ok(format!(
        "Command successfully processed in the system\nWork order {} was correctly scheduled into period {}",
        work_order_number,
        period.period_string()
    ).into_response())
    // TODO [ ] M
    // orchestrator.get_work_order(id)
}
