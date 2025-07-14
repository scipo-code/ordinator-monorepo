use std::sync::Arc;

use anyhow::Context;
use axum::Json;
use axum::debug_handler;
use axum::extract::Path;
use axum::extract::State;
use axum::response::Result;
use ordinator_contracts::AssetNames;
use ordinator_contracts::PeriodDto;
use ordinator_contracts::TotalSystemSolution;
use ordinator_contracts::WorkOrderNumberDto;
use ordinator_contracts::scheduler::SchedulerWorkOrderDto;
use ordinator_contracts::scheduler::WorkOrderSingleRowSimpleDto;
use ordinator_orchestrator::Asset;
use ordinator_orchestrator::Orchestrator;
use ordinator_orchestrator::StateLink;
use ordinator_orchestrator::WorkOrderNumber;

use crate::routes::api::AppError;

// This is a handler. Not a `Route` you should change that. Keep working.
#[debug_handler]
#[utoipa::path(
    get,
    path = "/scheduler/work_orders_with_scheduling/{asset}",

    tag = "Scheduler",
    params (
        ("asset" = AssetNames, Path),
    ),
    responses(
        (status = 200, body = WorkOrderSingleRowSimpleDto),
        (status = 404, body = AppError),
        (status = 500, body = AppError),
    )
)]
pub async fn get_scheduler_work_orders(
    State(_orchestrator): State<Arc<Orchestrator<TotalSystemSolution>>>,
    Path(asset): Path<AssetNames>,
) -> Result<Json<SchedulerWorkOrderDto>, AppError>
{
    // This should go into the handler, directory. There is no other way around it
    // REMEMBER: You should only wrap method calls that the Orchestrator exposes.
    //
    // WARN: You are beginning to feel drained again. You should grap something to
    // eat again.
    let asset = Asset::try_from(asset).map_err(|e| AppError::Anyhow(e.to_string()))?;
    let system_solution = _orchestrator
        .system_solutions
        .lock()
        .unwrap()
        .get(&asset.clone())
        .with_context(|| format!("Asset {:?} is not present in the ActorRegistry", &asset))
        .map_err(|e| AppError::Anyhow(e.to_string()))?
        .load();

    let scheduling_environment = _orchestrator.scheduling_environment.lock().unwrap();
    Ok(Json(
        SchedulerWorkOrderDto::try_from((asset.clone(), scheduling_environment, system_solution))
            .expect("This should never fail"),
    ))
    // TODO [ ] M
    // orchestrator.get_work_order(id)
}

/// This handler sends a `command` to the `StrategicActor` that a change to the
/// `UnloadingPoint` has occured. You need to send the `Actor` a message. should
/// you make the.
///
/// TODO [ ] Make an Essay
#[debug_handler]
#[utoipa::path(
    post,
    path = "/scheduler/{asset}/assign_work_order_to_period/{work_order_number}/{period}",
    tag = "Scheduler",
    params (
        ("asset" = AssetNames, Path),
        ("work_order_number" = WorkOrderNumberDto, Path),
        ("period" = PeriodDto, Path),
    ),
    responses(
        (status = 200, body = WorkOrderSingleRowSimpleDto),
        (status = 404, body = AppError),
        (status = 500, body = AppError),
    )
)]
pub async fn assign_work_order_to_period(
    State(_orchestrator): State<Arc<Orchestrator<TotalSystemSolution>>>,
    Path((asset, work_order_number_dto, period_dto)): Path<(
        AssetNames,
        WorkOrderNumberDto,
        PeriodDto,
    )>,
) -> Result<Json<SchedulerWorkOrderDto>, AppError>
{
    // This should go into the handler, directory. There is no other way around it
    // REMEMBER: You should only wrap method calls that the Orchestrator exposes.
    //
    // WARN: You are beginning to feel drained again. You should grap something to
    // eat again.
    let asset = Asset::try_from(asset).map_err(|e| AppError::Anyhow(e.to_string()))?;

    let scheduling_environment = _orchestrator.scheduling_environment.lock().unwrap();

    let work_order_number = WorkOrderNumber::from(work_order_number_dto);
    let period_option = scheduling_environment
        .time_environment
        .periods
        .iter()
        .find(|per| per.period_string() == period_dto.period_string);

    let period = match period_option {
        Some(period) => period,
        None => {
            return Err(AppError::Anyhow(format!(
                "Period: {:?}\nwas not found in the scheduling_environment\nDid you:\n1. use the pattern yyyy-Wxx-Wzz?\n2. Use the correct even or uneven week numbers correctly",
                period_dto
            )));
        }
    };

    scheduling_environment
        .work_orders
        .update_period_unloading_point(&work_order_number, period);

    // I hated that message structure when I first wrote it, now you have a lesson
    // to learn. So the [`StateLink`] is responsible for updating the `Actors` when
    // there has been a change to the scheduling_environment. The messages that
    // simply guide the search procedure should be handled the same way as
    // always.
    let state_link = StateLink::WorkOrders(vec![work_order_number]);

    // You are doing what you should have done a long time ago.
    _orchestrator
        .state_link_bus
        .lock()
        .unwrap()
        .broadcast(state_link);
    // Send a `StateLink` message to all Actors
    // TODO [x] 2025-07-14 make a broadcast channel.
    // It is very important that you get this right. You should make sure to use the
    // correct construct. I think
    _orchestrator.Ok(Json(
        SchedulerWorkOrderDto::try_from((asset.clone(), scheduling_environment, system_solution))
            .expect("This should never fail"),
    ))
    // TODO [ ] M
    // orchestrator.get_work_order(id)
}
