use std::sync::Arc;

use anyhow::Context;
use axum::Json;
use axum::body::Bytes;
use axum::debug_handler;
use axum::extract::Path;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::http::StatusCode;
use axum::http::header;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::response::Result;
use ordinator_contracts::AssetNames;
use ordinator_contracts::TotalSystemSolution;
use ordinator_contracts::scheduler::WorkOrderSingleRowSimpleDto;
use ordinator_orchestrator::Asset;
use ordinator_orchestrator::Orchestrator;
use ordinator_orchestrator::OrchestratorRequest;
use ordinator_orchestrator::Skill;
use ordinator_orchestrator::WorkOrderNumber;
use strum::IntoEnumIterator;

use crate::routes::api::AppError;

// TODO: Delete this function and replace with the other handler.
// pub async fn handle_orchestrator_message<Ss>(
//     State(orchestrator): State<Arc<Mutex<Orchestrator<Ss>>>>,
//     Json(reg): Json<OrchestratorRequest>,
// ) -> Result<Response, axum::Error>
// {
//     let mut orchestrator = orchestrator.lock().unwrap();
//     Ok(orchestrator.handle_orchestrator_request(reg).await?)
// }

pub async fn scheduler_excel_export(
    State(orchestrator): State<Arc<Orchestrator<TotalSystemSolution>>>,
    Path(asset): Path<AssetNames>,
) -> Result<Response>
{
    let asset = Asset::try_from(asset).map_err(|e| AppError::Anyhow(e.to_string()))?;

    let mut headers = HeaderMap::new();

    let (buffer, http_header) = orchestrator
        .export_xlsx_solution(asset)
        .expect("Could not export xlsx");

    headers.insert(
        header::CONTENT_TYPE,
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
            .parse()
            .unwrap(),
    );

    headers.insert(header::CONTENT_DISPOSITION, http_header.parse().unwrap());

    Ok((StatusCode::OK, headers, Bytes::from(buffer)).into_response())
}

#[utoipa::path(
    get,
    tag = "General",
    path = "/assets",
    responses((status = 200, body = [AssetNames]))
)]
pub async fn scheduler_asset_names() -> Response
{
    let asset_names = AssetNames::convert_to_asset_names();

    Json(asset_names).into_response()
}

#[utoipa::path(
    get,
    tag = "General",
    path = "/periods",
    responses((status = 200, body = [Vec<String>]))
)]
pub async fn periods(State(orchestrator): State<Arc<Orchestrator<TotalSystemSolution>>>)
-> Response
{
    let periods: Vec<_> = orchestrator
        .scheduling_environment
        .lock()
        .expect("Should never happen")
        .time_environment
        .periods
        .iter()
        .map(|e| e.period_string())
        .collect();

    Json(periods).into_response()
}

#[utoipa::path(
    get,
    tag = "General",
    path = "/days",
    responses((status = 200, body = [Vec<String>]))
)]
pub async fn days(State(orchestrator): State<Arc<Orchestrator<TotalSystemSolution>>>) -> Response
{
    let days: Vec<_> = orchestrator
        .scheduling_environment
        .lock()
        .expect("Should ever happen")
        .time_environment
        .days
        .iter()
        .map(|e| e.date.to_string())
        .collect();

    Json(days).into_response()
}

#[utoipa::path(
    get,
    tag = "General",
    path = "/work_order_info/{work_order_number}",

    params (
      ("work_order_number" = u64, Path)  
    ),
    responses((status = 200, body = [WorkOrderSingleRowSimpleDto]))
)]
pub async fn work_order_info(
    State(orchestrator): State<Arc<Orchestrator<TotalSystemSolution>>>,
    Path(work_order_number): Path<u64>,
) -> Result<Json<WorkOrderSingleRowSimpleDto>, AppError>
{
    let lock = orchestrator
        .scheduling_environment
        .lock()
        .expect("Should ever happen");
    let work_order = lock
        .work_orders
        .inner
        .get(&WorkOrderNumber(work_order_number))
        .with_context(||format!("{work_order_number} does not exist. Either:\n1. Typo in the WO number\n2. Wrong or old data\n3. Bug in the system: Call 004528433974")).map_err(|e| AppError::Anyhow(e.to_string()))?;

    let work_order_dto = WorkOrderSingleRowSimpleDto::from(work_order.clone());
    Ok(Json(work_order_dto))
}

#[utoipa::path(
    get,
    tag = "General",
    path = "/system_clock",
    responses((status = 200, body = String))
)]
pub async fn system_clock(
    State(orchestrator): State<Arc<Orchestrator<TotalSystemSolution>>>,
) -> Result<Response, AppError>
{
    let system_datetime = orchestrator
        .system_clock_tick_receiver
        .recv()
        .map_err(|e| AppError::Anyhow(e.to_string()))?
        .to_rfc3339();

    Ok(Json(system_datetime).into_response())
}

#[utoipa::path(get,
    tag = "General",
    path = "/resources",
    responses((status = 200,body = String))
)]
pub async fn resources(
    State(_orchestrator): State<Arc<Orchestrator<TotalSystemSolution>>>,
) -> Response
{
    let resources: Vec<_> = Skill::iter().map(|r| r.to_string()).collect();

    Json(resources).into_response()
}

// TODO: Refactor handlers to send OrchestratorRequest messages through Actors
// using DTO objects from the contracts crate instead of direct orchestrator calls.
pub async fn orchestrator_status(
    State(orchestrator): State<Arc<Orchestrator<TotalSystemSolution>>>,
) -> Result<Response>
{
    Ok(Json(orchestrator.actor_registries.lock().unwrap().len()).into_response())
}

pub async fn get_days(
    orchestrator: State<Arc<Orchestrator<TotalSystemSolution>>>,
) -> Result<Response>
{
    let json = Json(OrchestratorRequest::GetPeriods);
    orchestrator_helper(orchestrator, json).await
}

/// Route orchestrator requests through a helper function.
///
/// Sends an [`OrchestratorRequest`] to the orchestrator and returns the response.
#[debug_handler]
pub async fn orchestrator_helper(
    State(orchestrator): State<Arc<Orchestrator<TotalSystemSolution>>>,
    Json(orchestrator_request): Json<OrchestratorRequest>,
) -> Result<Response>
{
    let response = orchestrator
        .handle(orchestrator_request)
        .await
        .unwrap();

    Ok(Json(response).into_response())
}
