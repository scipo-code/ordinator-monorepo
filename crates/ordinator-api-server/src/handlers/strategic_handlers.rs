use std::sync::Arc;

use anyhow::Context;
use axum::Json;
use axum::debug_handler;
use axum::extract::Path;
use axum::extract::State;
use axum::response::Result;
use ordinator_contracts::AssetNames;
use ordinator_contracts::TotalSystemSolution;
use ordinator_contracts::scheduler::SchedulerWorkOrderDto;
use ordinator_contracts::scheduler::WorkOrderSingleRowSimpleDto;
use ordinator_orchestrator::Asset;
use ordinator_orchestrator::Orchestrator;

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
