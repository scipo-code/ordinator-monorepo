use std::collections::BTreeMap;
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
use ordinator_orchestrator::ProjectInterface;
use ordinator_orchestrator::ProjectRequestMessage;
use ordinator_orchestrator::ProjectStatusMessage;
use ordinator_orchestrator::WorkOrderNumber;
use serde::Serialize;
use tracing::info;
use ts_rs::TS;
use utoipa::ToSchema;

use crate::routes::api::AppError;

// Each handler should construct a specific message and send it via the orchestrator.
// Consider consolidating to pass a single RequestMessage with a corresponding StatusMessage.
#[utoipa::path(
    get,
    tag = "Scheduler",
    path = "/tactical_algorithm_status",
    responses((status = 200, body = [Vec<String>]))
)]
pub async fn status<Ss>(
    State(orchestrator): State<Arc<Orchestrator<Ss>>>,
    Path(asset): Path<AssetNames>,
) -> Result<Response, AppError>
where
    Ss: SystemSolutions,
{
    let asset = Asset::try_from(asset).map_err(|e| AppError::Anyhow(e.to_string()))?;

    let message = ProjectRequestMessage::Status(ProjectStatusMessage::General);

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
//     tag = "Daily",
//     path = "/{asset}/{supervisor_id}",
//     params (
//         ("asset" = AssetNames, Path),
//         ("supervisor_id" = String, Path),
//     ),
//     responses(
//         (status = 200, body = DailyResponseMessageDto),
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
    Path(asset): Path<AssetNames>,
) -> Result<Response, AppError>
where
    Ss: SystemSolutions,
{
    let asset = Asset::try_from(asset).map_err(|e| AppError::Anyhow(e.to_string()))?;

    let tactical_days = orchestrator
        .system_solutions
        .lock()
        .unwrap_or_else(|_| panic!("Could not lock the SystemSolution for Asset: {}", &asset))
        .get(&asset)
        .with_context(|| format!("SystemSolution for Asset: {} does not exist", &asset))
        .map_err(|e| AppError::Anyhow(e.to_string()))?
        .load()
        .tactical_actor_solution()
        .map_err(|_| AppError::Anyhow(format!("No ProjectSolution exists for Asset: {}", &asset)))?
        .all_scheduled_tasks();

    Ok(Json(tactical_days).into_response())
}

#[utoipa::path(
    post,
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
    // TODO: Use `asset` for authentication.
    Path((asset, work_order_number)): Path<(AssetNames, WorkOrderNumber)>,
    Json(basic_start_date_dto): Json<NaiveDateDto>,
) -> Result<Response, AppError>
where
    Ss: SystemSolutions,
{
    let _asset = Asset::try_from(asset).map_err(|e| AppError::Anyhow(e.to_string()))?;

    let basic_start_date: NaiveDate = basic_start_date_dto
        .try_into()
        .map_err(|e: ParseError| AppError::Anyhow(e.to_string()))?;
    let mut scheduling_environment_lock = orchestrator.scheduling_environment.lock().unwrap();

    let materials_to_periods = &scheduling_environment_lock
        .material_repo
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
        // TODO: Separate work order modification code from regular code path.
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

#[derive(Serialize, TS, ToSchema)]
#[ts(export, export_to = "../../../static_files/packages/shared/src/types/")]
pub struct DailyLoad
{
    work: f64,
    day: NaiveDateDto,
}

#[derive(Serialize, TS, ToSchema)]
#[ts(export, export_to = "../../../static_files/packages/shared/src/types/")]
pub struct DailyLoadingDto
{
    resources: BTreeMap<String, Vec<DailyLoad>>,
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
    Path(asset): Path<AssetNames>,
) -> Result<Response, AppError>
where
    Ss: SystemSolutions,
{
    let asset = Asset::try_from(asset).map_err(|e| AppError::Anyhow(e.to_string()))?;

    let tactical_days = orchestrator
        .system_solutions
        .lock()
        .unwrap_or_else(|_| panic!("Could not lock the SystemSolution for Asset: {}", &asset))
        .get(&asset)
        .with_context(|| format!("SystemSolution for Asset: {} does not exist", &asset))
        .map_err(|e| AppError::Anyhow(e.to_string()))?
        .load()
        .tactical_actor_solution()
        .map_err(|_| AppError::Anyhow(format!("No ProjectSolution exists for Asset: {}", &asset)))?
        .tactical_loadings();

    let days = orchestrator
        .scheduling_environment
        .lock()
        .unwrap_or_else(|_| panic!("Could not lock the SystemSolution for Asset: {}", &asset))
        .time_environment
        .days
        .clone();

    info!(target: "developer", tactical_days = ?tactical_days);

    if tactical_days
        .values()
        .all(|work_vec| work_vec.len() == days.len())
    {
        let daily_loadings = tactical_days
            .into_iter()
            .map(|(resource, work_vec)| {
                let daily_loads = work_vec
                    .into_iter()
                    .zip(days.iter())
                    .map(|(work, day)| DailyLoad {
                        work: work.to_f64(),
                        day: NaiveDateDto::from(day.date),
                    })
                    .collect();
                (resource.to_string(), daily_loads)
            })
            .collect::<BTreeMap<_, Vec<_>>>();

        Ok(Json(DailyLoadingDto {
            resources: daily_loadings,
        })
        .into_response())
    } else {
        Err(AppError::Anyhow(
            "Days and dailyloading does not match".to_string(),
        ))
    }
}
