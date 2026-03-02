use std::str::FromStr;
use std::sync::Arc;

use anyhow::Context;
use anyhow::anyhow;
use axum::Json;
use axum::debug_handler;
use axum::extract::Path;
use axum::extract::State;
use axum::response::Result;
use axum_extra::extract::Query;
use chrono::DateTime;
use ordinator_contracts::AssetNames;
use ordinator_contracts::DateTimeDto;
use ordinator_contracts::NaiveDateDto;
use ordinator_contracts::TotalSystemSolution;
use ordinator_contracts::WorkOrderNumberDto;
use ordinator_contracts::daily::DailyAllAvailableTechnicians;
use ordinator_contracts::daily::DailyMainTableDto;
use ordinator_contracts::daily::DailyResourcesDto;
use ordinator_contracts::daily::DailyResponseMessageDto;
use ordinator_orchestrator::Asset;
use ordinator_orchestrator::Orchestrator;
use ordinator_orchestrator::Skill;
use ordinator_orchestrator::StartError;
use ordinator_orchestrator::StateLink;
use ordinator_orchestrator::DailyRequestMessage;
use ordinator_orchestrator::DailyStatusMessage::General;
use ordinator_orchestrator::WorkOrderNumber;
use serde::Deserialize;
use serde::Serialize;
use ts_rs::TS;
use utoipa::IntoParams;
use utoipa::ToSchema;

use crate::routes::api::AppError;

#[debug_handler]
#[utoipa::path(
    get,
    tag = "Daily",
    path = "/{asset}/{daily_id}",
    params (
        ("asset" = AssetNames, Path),
        ("daily_id" = String, Path),
    ),
    responses(
        (status = 200, body = DailyResponseMessageDto),
        (status = 404, body = AppError),
        (status = 500, body = AppError),
    )
)]
pub async fn status(
    State(orchestrator): State<Arc<Orchestrator<TotalSystemSolution>>>,
    Path((asset, daily_id)): Path<(AssetNames, String)>,
) -> Result<Json<DailyResponseMessageDto>, AppError>
{
    let asset = Asset::try_from(asset)
        .map_err(|e| AppError::Anyhow(e.to_string() + "Could not parse the Asset parameter"))?;

    let lock = orchestrator.actor_registries.lock().unwrap();
    let daily_agent_senders = &lock
        .get(&asset)
        .with_context(|| format!("Asset {asset} is not present in the ActorRegistry"))
        .unwrap()
        .daily_agent_senders;

    let daily_id = daily_agent_senders
        .keys()
        .find(|e| e.0 == daily_id)
        .ok_or(AppError::Anyhow(
            anyhow!("Daily Not found").to_string(),
        ))?;

    let communication = daily_agent_senders
        .get(daily_id)
        .ok_or(AppError::Anyhow(
            anyhow!("Daily not found").to_string(),
        ))?;

    communication
        .from_agent(DailyRequestMessage::Status(General))
        .unwrap();

    Ok(Json(communication.from_actor().into()))
}

#[debug_handler]
#[utoipa::path(
    get,
    path = "/technician_availability/{asset}/{daily_id}",
    tag = "Daily",
    params (
        ("asset" = AssetNames, Path),
        ("daily_id" = String, Path),
    ),
    responses(
        (status = 200, body = DailyAllAvailableTechnicians),
        (status = 404, body = AppError),
        (status = 500, body = AppError),
    )
)]
pub async fn technician_availability(
    State(orchestrator): State<Arc<Orchestrator<TotalSystemSolution>>>,
    // TODO: Use `_daily_id` when additional filtering is needed
    Path((asset, _daily_id)): Path<(AssetNames, String)>,
) -> Result<Json<DailyAllAvailableTechnicians>, AppError>
{
    let asset = Asset::try_from(asset)
        .map_err(|e| AppError::Anyhow(e.to_string() + "Could not parse the Asset parameter"))?;
    let daily_all_available_technicians: DailyAllAvailableTechnicians = orchestrator
        .scheduling_environment
        .lock()
        .expect("Cannot lock SchedulingEnvironment")
        .worker_environment
        .actor_specification
        .get(&asset)
        .context("ActorSpecification not available for the Asset")
        .map_err(|e| AppError::Anyhow(e.to_string()))?
        .technician_availability()
        .into();

    Ok(Json(daily_all_available_technicians))
}

#[debug_handler]
#[utoipa::path(
    get,
    path = "/all_technicians/{asset}/{daily_id}",
    tag = "Daily",
    params (
        ("asset" = AssetNames, Path),
        ("daily_id" = String, Path),
    ),
    responses(
        (status = 200, body = DailyResourcesDto),
        (status = 404, body = AppError),
        (status = 500, body = AppError),
    )
)]
pub async fn all_technicians(
    State(orchestrator): State<Arc<Orchestrator<TotalSystemSolution>>>,
    // TODO: Use `_daily_id` when additional filtering is needed
    Path((asset, _daily_id)): Path<(AssetNames, String)>,
) -> Result<Json<DailyResourcesDto>, AppError>
{
    let asset = Asset::try_from(asset)
        .map_err(|e| AppError::Anyhow(e.to_string() + "Could not parse the Asset parameter"))?;

    let daily_resources: DailyResourcesDto = orchestrator
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
        .daily
        .as_ref()
        .ok_or(AppError::Anyhow(
            "DailySolution not available. Likely due to the Daily not being instantiated"
                .to_string(),
        ))?
        .inner()
        .clone()
        .into();

    Ok(Json(daily_resources))
}

#[derive(Deserialize, IntoParams, ToSchema)]
pub struct MainTableQueryParams
{
    #[serde(default)]
    pub day: NaiveDateDto,
}

#[debug_handler]
#[utoipa::path(
    get,
    path = "/daily_main_table/{asset}/{daily_id}",
    tag = "Daily",
    params (
        ("asset" = AssetNames, Path),
        ("daily_id" = String, Path),
        MainTableQueryParams,
    ),
    responses(
        (status = 200, body = DailyMainTableDto),
        (status = 404, body = AppError),
        (status = 500, body = AppError),
    )
)]
pub async fn daily_main_table(
    State(orchestrator): State<Arc<Orchestrator<TotalSystemSolution>>>,
    // TODO: Use `_daily_id` when additional filtering is needed
    Path((asset, _daily_id)): Path<(AssetNames, String)>,
    Query(query): Query<MainTableQueryParams>,
) -> Result<Json<DailyMainTableDto>, AppError>
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

    let mut daily_main_table_dto =
        DailyMainTableDto::try_from((work_orders, system_solution, time_environment))
            .with_context(|| {
                format!("DailyMainTable could not be constructed for {_daily_id}")
            })
            .map_err(|e| {
                AppError::Anyhow(e.to_string() + "could not create the DailyMainTableDto")
            })?;

    let query_day = query.day;

    if !query_day.0.is_empty() {
        daily_main_table_dto
            .days
            .retain(|day_key, _| day_key == &query_day);
    };

    Ok(Json(daily_main_table_dto))
}

// TODO [ ] - assert that number is equal to the number of assignments.
#[derive(Serialize, Deserialize, ToSchema)]
pub struct WorkOrderActivityToTechnicianDto
{
    #[schema(example = 2100001234)]
    work_order_number: WorkOrderNumberDto,
    #[schema(example = 20)]
    activity_number: u64,
    #[schema(example = "[l1113333, l1112222]")]
    technicians: Vec<String>,
}

// TODO: Verify that the number of technicians matches the number of assignments
#[debug_handler]
#[utoipa::path(
    patch,
    path = "/assign_to_technicians/{asset}/{daily_id}",
    tag = "Daily",
    description = "This endpoint is for assigning a technicians to a work order activity.",
    params (
        ("asset" = AssetNames, Path),
        ("daily_id" = String, Path),
    ),
    request_body(
        content = WorkOrderActivityToTechnicianDto,
    ),
    responses(
        (status = 201, description = "Work order activity assigned to technicians"),
        (status = 404, body = AppError),
        (status = 500, body = AppError),
    )
)]
pub async fn assign_to_technicians(
    State(orchestrator): State<Arc<Orchestrator<TotalSystemSolution>>>,
    // TODO: Use `_daily_id` when additional filtering is needed
    Path((asset, _daily_id)): Path<(AssetNames, String)>,
    Json(payload): Json<WorkOrderActivityToTechnicianDto>,
) -> Result<(), AppError>
{
    let WorkOrderActivityToTechnicianDto {
        work_order_number,
        activity_number,
        technicians,
    } = payload;

    let work_order_number = WorkOrderNumber::from(work_order_number);

    let mut scheduling_environment_lock = orchestrator
        .scheduling_environment
        .lock()
        .map_err(|e| AppError::Anyhow(e.to_string()))?;

    let days = &scheduling_environment_lock.time_environment.days;
    let periods = &scheduling_environment_lock.time_environment.periods;
    let asset: Asset = asset
        .try_into()
        .map_err(|_| AppError::Anyhow("Could not convert into error".to_string()))?;

    let actor_specification = &scheduling_environment_lock
        .worker_environment
        .actor_specification
        .get(&asset)
        .ok_or(AppError::Anyhow(
            "ActorSpecification not found for Asset".to_string(),
        ))?;

    // Make the assignment; StateLink bus handles interaction with multiple parties
    let technician = actor_specification
        .operational()
        .iter()
        .filter(|e| technicians.contains(e.0))
        .map(|e| e.0.clone())
        .collect::<Vec<_>>();

    let material_to_period = &scheduling_environment_lock.material_repo.material_to_period;

    let forced_work_order = scheduling_environment_lock
        .work_orders
        .inner
        .get(&work_order_number)
        .ok_or(AppError::Anyhow("WorkOrder not found".to_string()))?
        .forced_work_order(periods, days, material_to_period)
        .map_err(|e| AppError::Anyhow(e.to_string()))?;

    // TODO: Reimplement assignment through SchedulingHypergraph
    let _ = (&forced_work_order, &activity_number, &technician);
    return Err(AppError::Anyhow(
        "Assignment through SchedulingEnvironment not yet reimplemented".to_string(),
    ));

    orchestrator
        .state_link_bus
        .lock()
        .unwrap()
        .broadcast(StateLink::WorkOrders(vec![work_order_number]));

    Ok(())
}

#[derive(Serialize, Deserialize, ToSchema, TS)]
#[ts(export, export_to = "../../../static_files/packages/shared/src/types/")]
pub struct CreateTechnicianDto
{
    #[schema(example = "l1112233")]
    id: String,
    #[schema(example = "[\"MTN-MECH\"]")]
    resources_string: Vec<String>,
    #[schema(example = "\"2025-01-01T07:00:00Z\"")]
    start: DateTimeDto,
    #[schema(example = "\"2025-01-01T07:00:00Z\"")]
    finish: DateTimeDto,
}

#[debug_handler]
#[utoipa::path(
    post,
    path = "/add_technician/{asset}/{daily_id}",
    tag = "Daily",
    description = "This endpoint is for assigning a technicians to a work order activity.",
    params (
        ("asset" = AssetNames, Path),
        ("daily_id" = String, Path),
    ),
    request_body(
        content = CreateTechnicianDto,
    ),
    responses(
        (status = 201, description = "Work order activity assigned to technicians"),
        (status = 404, body = AppError),
        (status = 500, body = AppError),
    )
)]
pub async fn add_technician(
    State(orchestrator): State<Arc<Orchestrator<TotalSystemSolution>>>,
    // TODO: Use `_daily_id` when additional filtering is needed
    Path((asset, _daily_id)): Path<(AssetNames, String)>,
    Json(payload): Json<CreateTechnicianDto>,
) -> Result<Json<String>, AppError>
{
    let CreateTechnicianDto {
        id,
        resources_string,
        start,
        finish,
    } = payload;

    let start_date = DateTime::try_from(start)
        .map_err(|e| AppError::Anyhow(format!("Error at start date: {}.", e)))?;
    let finish_date = DateTime::try_from(finish)
        .map_err(|e| AppError::Anyhow(format!("Error at finish date: {}.", e)))?;

    let mut resources = vec![];
    for resource_string in resources_string {
        let resource =
            Skill::from_str(&resource_string).map_err(|e| AppError::Anyhow(e.to_string()))?;

        resources.push(resource);
    }

    // TODO: Verify that the worker is not already present before adding
    let asset = Asset::try_from(asset)
        .map_err(|_e| AppError::Anyhow("Incorrect asset name".to_string()))?;

    let actor_composite_id = orchestrator
        .scheduling_environment
        .lock()
        .unwrap()
        .worker_environment
        .actor_specification
        .get_mut(&asset)
        .context("No ActorSpecification available to Asset")
        .map_err(|e| AppError::Anyhow(e.to_string()))?
        .add_operational(
            &id,
            vec![asset.clone()],
            resources.clone(),
            start_date,
            finish_date,
        )
        .map_err(|e| AppError::Anyhow(e.to_string()))?;

    if let Err(started) = orchestrator.start_operational_actor(&actor_composite_id) {
        match started {
            StartError::AlreadyRunning => {
                return Ok(Json(format!(
                    "Technician {} already exists and is running",
                    id
                )));
            }
            StartError::CouldNotConstruct => {
                return Err(AppError::Anyhow("could not construct Actor".to_string()));
            }
            StartError::CouldNotCreateDependencies => {
                return Err(AppError::Anyhow(
                    "could not construct Actor dependencies".to_string(),
                ));
            }
        }
    }
    Ok(Json(format!(
        "Technician {} successfully created and started",
        id
    )))
}
