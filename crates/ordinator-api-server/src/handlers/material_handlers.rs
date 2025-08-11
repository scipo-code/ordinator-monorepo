use std::fmt::Display;
use std::sync::Arc;

use axum::Json;
use axum::debug_handler;
use axum::extract::Path;
use axum::extract::State;
use ordinator_contracts::AssetNames;
use ordinator_contracts::TotalSystemSolution;
use ordinator_contracts::WorkOrderNumberDto;
use ordinator_orchestrator::Orchestrator;
use ordinator_orchestrator::StateLink;
use ordinator_orchestrator::WorkOrderNumber;
use serde::Deserialize;
use ts_rs::TS;
use utoipa::ToSchema;

use crate::routes::api::AppError;

#[derive(Deserialize, ToSchema, TS, Debug)]
#[ts(export, export_to = "../../../static_files/packages/shared/src/types/")]
pub struct MaterialCheck(bool);

impl Display for MaterialCheck
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        write!(f, "{}", self.0)
    }
}

#[debug_handler]
#[utoipa::path(
    patch,
    path = "/check_material/{asset}/{work_order_number}/",
    tag = "Material Clerk",
    description = "Check that material is present. This will cause a update in status code SMAT",
    params (
        ("asset" = AssetNames, Path),
        ("work_order_number" = WorkOrderNumberDto, Path),
    ),
    request_body(content = MaterialCheck, description = "json true or false", content_type = "application/json", example = json!(true)),
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
    Path((_asset, work_order_number)): Path<(AssetNames, WorkOrderNumberDto)>,
    Json(checked): Json<MaterialCheck>,
) -> Result<String, AppError>
{
    let work_order_number = WorkOrderNumber(work_order_number.0);
    orchestrator
        .scheduling_environment
        .lock()
        .unwrap()
        .work_orders
        .update_material_checked(&work_order_number, checked.0)
        .map_err(|e| AppError::Anyhow(e.to_string()))?;

    orchestrator
        .state_link_bus
        .lock()
        .unwrap()
        .broadcast(StateLink::WorkOrders(vec![work_order_number]));

    Ok(format!(
        "Off shore material status for work order {} set to {:?}",
        work_order_number, checked
    ))
}
