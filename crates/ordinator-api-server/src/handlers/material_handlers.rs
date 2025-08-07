use std::sync::Arc;

use axum::debug_handler;
use axum::extract::Path;
use axum::extract::State;
use ordinator_contracts::AssetNames;
use ordinator_contracts::TotalSystemSolution;
use ordinator_contracts::WorkOrderNumberDto;
use ordinator_orchestrator::Orchestrator;
use ordinator_orchestrator::StateLink;
use ordinator_orchestrator::WorkOrderNumber;

use crate::routes::api::AppError;

#[debug_handler]
#[utoipa::path(
    patch,
    path = "/check_material/{asset}/{work_order_number}/{checked}",
    tag = "Material Clerk",
    description = "Check that material is present. This will cause a update in status code SMAT",
    params (
        ("asset" = AssetNames, Path),
        ("work_order_number" = WorkOrderNumberDto, Path),
        ("checked" = bool, Path),
    ),
    responses(
        (status = 201, description = "Work order materials received off-shore have been confirmed"),
        (status = 404, body = AppError),
        (status = 500, body = AppError),
    )
)]
pub async fn check_material(
    State(orchestrator): State<Arc<Orchestrator<TotalSystemSolution>>>,
    // TODO [ ]
    // The `_supervisor_id` should be used in the future when we have additional
    Path((_asset, work_order_number, checked)): Path<(AssetNames, WorkOrderNumberDto, bool)>,
) -> Result<String, AppError>
{
    let work_order_number = WorkOrderNumber(work_order_number.0);
    orchestrator
        .scheduling_environment
        .lock()
        .unwrap()
        .work_orders
        .update_material_checked(&work_order_number, checked)
        .map_err(|e| AppError::Anyhow(e.to_string()))?;

    orchestrator
        .state_link_bus
        .lock()
        .unwrap()
        .broadcast(StateLink::WorkOrders(vec![work_order_number]));

    Ok(format!(
        "Off shore material status for work order {} set to {}",
        work_order_number, checked
    ))
}
