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
use ordinator_contracts::supervisor::SupervisorAllAvailableTechnicians;
use ordinator_contracts::supervisor::SupervisorMainTableDto;
use ordinator_contracts::supervisor::SupervisorResourcesDto;
use ordinator_contracts::supervisor::SupervisorResponseMessageDto;
use ordinator_orchestrator::Asset;
use ordinator_orchestrator::Orchestrator;
use ordinator_orchestrator::Resources;
use ordinator_orchestrator::StartError;
use ordinator_orchestrator::StateLink;
use ordinator_orchestrator::SupervisorRequestMessage;
use ordinator_orchestrator::SupervisorStatusMessage::General;
use ordinator_orchestrator::WorkOrderNumber;
// This again means that you are doing something wrong. You are implementing the
// methods in the wrong place
// This should be moved away from here.
use serde::Deserialize;
use serde::Serialize;
use ts_rs::TS;
use utoipa::IntoParams;
use utoipa::ToSchema;

use crate::routes::api::AppError;

#[debug_handler]
#[utoipa::path(
    get,
    tag = "Supervisor",
    path = "/{asset}/{supervisor_id}",
    params (
        ("asset" = AssetNames, Path),
        ("supervisor_id" = String, Path),
    ),
    responses(
        (status = 200, body = SupervisorResponseMessageDto),
        (status = 404, body = AppError),
        (status = 500, body = AppError),
    )
)]
pub async fn status(
    State(orchestrator): State<Arc<Orchestrator<TotalSystemSolution>>>,
    Path((asset, supervisor_id)): Path<(AssetNames, String)>,
) -> Result<Json<SupervisorResponseMessageDto>, AppError>
{
    let asset = Asset::try_from(asset)
        .map_err(|e| AppError::Anyhow(e.to_string() + "Could not parse the Asset parameter"))?;

    let lock = orchestrator.actor_registries.lock().unwrap();
    let supervisor_agent_senders = &lock
        .get(&asset)
        .with_context(|| format!("Asset {asset} is not present in the ActorRegistry"))
        .unwrap()
        .supervisor_agent_senders;

    let supervisor_id = supervisor_agent_senders
        .keys()
        .find(|e| e.0 == supervisor_id)
        .ok_or(AppError::Anyhow(
            anyhow!("Supervisor Not found").to_string(),
        ))?;

    let communication = supervisor_agent_senders
        .get(supervisor_id)
        .ok_or(AppError::Anyhow(
            anyhow!("Supervisor not found").to_string(),
        ))?;

    communication
        .from_agent(SupervisorRequestMessage::Status(General))
        .unwrap();

    Ok(Json(communication.from_actor().into()))
}

#[debug_handler]
#[utoipa::path(
    get,
    path = "/technician_availability/{asset}/{supervisor_id}",
    tag = "Supervisor",
    params (
        ("asset" = AssetNames, Path),
        ("supervisor_id" = String, Path),
    ),
    responses(
        (status = 200, body = SupervisorAllAvailableTechnicians),
        (status = 404, body = AppError),
        (status = 500, body = AppError),
    )
)]
pub async fn technician_availability(
    State(orchestrator): State<Arc<Orchestrator<TotalSystemSolution>>>,
    // TODO [ ]
    // The `_supervisor_id` should be used in the future when we have additional
    Path((asset, _supervisor_id)): Path<(AssetNames, String)>,
) -> Result<Json<SupervisorAllAvailableTechnicians>, AppError>
{
    let asset = Asset::try_from(asset)
        .map_err(|e| AppError::Anyhow(e.to_string() + "Could not parse the Asset parameter"))?;
    let supervisor_all_available_technicians: SupervisorAllAvailableTechnicians = orchestrator
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

    Ok(Json(supervisor_all_available_technicians))
}

#[debug_handler]
#[utoipa::path(
    get,
    path = "/all_technicians/{asset}/{supervisor_id}",
    tag = "Supervisor",
    params (
        ("asset" = AssetNames, Path),
        ("supervisor_id" = String, Path),
    ),
    responses(
        (status = 200, body = SupervisorResourcesDto),
        (status = 404, body = AppError),
        (status = 500, body = AppError),
    )
)]
pub async fn all_technicians(
    State(orchestrator): State<Arc<Orchestrator<TotalSystemSolution>>>,
    // TODO [ ]
    // The `_supervisor_id` should be used in the future when we have additional
    Path((asset, _supervisor_id)): Path<(AssetNames, String)>,
) -> Result<Json<SupervisorResourcesDto>, AppError>
{
    // let lock = orchestrator.actor_registries.lock().unwrap();
    // let asset = Asset::try_from(asset).map_err(|e|
    // AppError::Anyhow(e.to_string()))?;
    let asset = Asset::try_from(asset)
        .map_err(|e| AppError::Anyhow(e.to_string() + "Could not parse the Asset parameter"))?;

    let supervisor_resources: SupervisorResourcesDto = orchestrator
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
        .supervisor
        .as_ref()
        .ok_or(AppError::Anyhow(
            "SupervisorSolution not available. Likely due to the Supervisor not being instantiated"
                .to_string(),
        ))?
        .inner()
        .clone()
        .into();

    // ISSUE #000
    // This code should be used for the Command part of the CQRS pattern
    // let supervisor_agent_senders = &lock
    //     .get(&asset.clone())
    //     .with_context(|| format!("Asset {asset:?} is not present in the
    // ActorRegistry"))     .unwrap()
    //     .supervisor_agent_senders;

    // let supervisor_id = supervisor_agent_senders
    //     .keys()
    //     .find(|e| e.0 == supervisor_id)
    //     .ok_or(AppError::Anyhow(
    //         anyhow!("Supervisor Not found").to_string(),
    //     ))?;

    // let communication = supervisor_agent_senders
    //     .get(supervisor_id)
    //     .ok_or(AppError::Anyhow(
    //         anyhow!("Supervisor not found").to_string(),
    //     ))?;

    // communication
    //     .from_agent(SupervisorRequestMessage::Status(General))
    //     .unwrap();

    Ok(Json(supervisor_resources))
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
    path = "/supervisor_main_table/{asset}/{supervisor_id}",
    tag = "Supervisor",
    params (
        ("asset" = AssetNames, Path),
        ("supervisor_id" = String, Path),
        MainTableQueryParams,
    ),
    responses(
        (status = 200, body = SupervisorMainTableDto),
        (status = 404, body = AppError),
        (status = 500, body = AppError),
    )
)]
pub async fn supervisor_main_table(
    State(orchestrator): State<Arc<Orchestrator<TotalSystemSolution>>>,
    // TODO [ ]
    // The `_supervisor_id` should be used in the future when we have additional
    Path((asset, _supervisor_id)): Path<(AssetNames, String)>,
    Query(query): Query<MainTableQueryParams>,
) -> Result<Json<SupervisorMainTableDto>, AppError>
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

    let mut supervisor_main_table_dto =
        SupervisorMainTableDto::try_from((work_orders, system_solution, time_environment))
            .with_context(|| {
                format!("SupervisorMainTable could not be constructed for {_supervisor_id}")
            })
            .map_err(|e| {
                AppError::Anyhow(e.to_string() + "could not create the SupervisorMainTableDto")
            })?;
    // let lock = orchestrator.actor_registries.lock().unwrap();
    //

    let query_day = query.day;

    if !query_day.0.is_empty() {
        supervisor_main_table_dto
            .days
            .retain(|day_key, _| day_key == &query_day);
    };

    Ok(Json(supervisor_main_table_dto))
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
#[debug_handler]
#[utoipa::path(
    patch,
    path = "/assign_to_technicians/{asset}/{supervisor_id}",
    tag = "Supervisor",
    description = "This endpoint is for assigning a technicians to a work order activity.",
    params (
        ("asset" = AssetNames, Path),
        ("supervisor_id" = String, Path),
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
    // TODO [ ]
    // The `_supervisor_id` should be used in the future when we have additional
    Path((asset, _supervisor_id)): Path<(AssetNames, String)>,
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

    // Here you simply have to make the assignment. The Bus::<StateLink> will have
    // the interation with multiple people.
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

    scheduling_environment_lock
        .assignments
        .make_assignment_for_technician(
            work_order_number,
            &forced_work_order,
            &activity_number,
            &technician,
        )
        .map_err(|e| AppError::Anyhow(e.to_string()))?;

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
    path = "/add_technician/{asset}/{supervisor_id}",
    tag = "Supervisor",
    description = "This endpoint is for assigning a technicians to a work order activity.",
    params (
        ("asset" = AssetNames, Path),
        ("supervisor_id" = String, Path),
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
    // TODO [ ]
    // The `_supervisor_id` should be used in the future when we have additional
    Path((asset, _supervisor_id)): Path<(AssetNames, String)>,
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
        .map_err(|e| AppError::Anyhow(format!("Error at start date: {}.", e.to_string())))?;
    let finish_date = DateTime::try_from(finish)
        .map_err(|e| AppError::Anyhow(format!("Error at finish date: {}.", e.to_string())))?;

    let mut resources = vec![];
    for resource_string in resources_string {
        let resource =
            Resources::from_str(&resource_string).map_err(|e| AppError::Anyhow(e.to_string()))?;

        resources.push(resource);
    }
    // This can only be created if there is somekind of... You need clear boundaries
    // here. You should check that the Worker is not already present.

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

// _ISSUE_ #000 means unassigned
// TODO [ ] ISSUE #000
// You should craft the needed requests here. You should not be working on the
// Making a general function to handle every type of request to each actor, is
// a good idea. You should make this after the system is up and running.
// pub async fn handle_supervisor_request<Ss>(
//     State(orchestrator): State<Arc<Mutex<Orchestrator<Ss>>>>,
//     supervisor_request: SupervisorRequest,
// ) -> Result<HttpResponse, actix_web::Error>
// where
//     Ss: SystemSolutionTrait,
// {
//     event!(Level::INFO, supervisor_request = ?supervisor_request);
//     let supervisor_agent_addrs = match
// self.agent_registries.get(&supervisor_request.asset) {
//         Some(agent_registry) => &agent_registry.supervisor_agent_senders,
//         None => {
//             return Ok(HttpResponse::BadRequest()
//                 .json("SUPERVISOR: SUPERVISOR AGENT NOT INITIALIZED FOR THE
// ASSET"));         }
//     };
//     let supervisor_agent_addr = supervisor_agent_addrs
//                 .iter()
//                 .find(|(id, _)| id.0 ==
// supervisor_request.supervisor.to_string())                 .expect("This will
// error at somepoint you will need to handle if you have added additional
// supervisors")                 .1;

//     // This was the reason that we wanted the tokio runtime.
//     supervisor_agent_addr
//         .sender
//         .send(crate::agents::ActorMessage::Actor(
//             supervisor_request.supervisor_request_message,
//         ))
//         .map_err(actix_web::error::ErrorInternalServerError)?;

//     let response = supervisor_agent_addr
//         .receiver
//         .recv()
//         .map_err(actix_web::error::ErrorInternalServerError)?
//         .map_err(actix_web::error::ErrorInternalServerError)?;

//     let supervisor_response =
// SupervisorResponse::new(supervisor_request.asset, response);

//     let system_responses = SystemResponses::Supervisor(supervisor_response);
//     Ok(HttpResponse::Ok().json(system_responses))
// }
