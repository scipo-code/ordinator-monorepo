use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::HashSet;
use std::string::String;

use anyhow::Context;
use ordinator_scheduling_environment::time_environment::TimeEnvironment;
use ordinator_scheduling_environment::work_order::WorkOrder;
use ordinator_scheduling_environment::work_order::WorkOrders;
use ordinator_scheduling_environment::work_order::operation::ActivityNumber;
use ordinator_scheduling_environment::worker_environment::IdString;
use ordinator_scheduling_environment::worker_environment::availability::Availability;
use ordinator_scheduling_environment::worker_environment::resources::Resources;
use ordinator_supervisor_actor::algorithm::supervisor_solution::SupervisorSolution;
use ordinator_supervisor_actor::messages::SupervisorResponseMessage;
use ordinator_supervisor_actor::messages::responses::SupervisorResponseStatus;
use serde::Serialize;
use ts_rs::TS;
use utoipa::ToSchema;

use crate::IdDto;
use crate::IdStringDto;
use crate::MaterialStatusDto;
use crate::NaiveDateDto;
use crate::TotalSystemSolution;
use crate::WorkOrderActivityDto;
use crate::WorkOrderNumberDto;

#[derive(ToSchema, Serialize)]
pub struct SupervisorResourcesDto
{
    all_technicians: BTreeSet<IdDto>,
    assigned_activities: BTreeMap<IdStringDto, WorkOrderActivityDto>,
}

#[derive(ToSchema, Serialize, Debug, TS)]
#[ts(export, export_to = "../../../static_files/packages/shared/src/types/")]
pub struct SupervisorMainTableDto
{
    pub days: BTreeMap<NaiveDateDto, DaySubtable>,
}

#[derive(ToSchema, Debug, Serialize, TS)]
#[ts(export, export_to = "../../../static_files/packages/shared/src/types/")]
pub struct DaySubtable
{
    // Each person should be mentioned once.
    work_order_activities_per_work_center: HashMap<String, Vec<WorkOrderSupervisorRow>>,
}

type Area = Option<String>;
type Permit = Option<String>;
type Description = String;
type Icc = Option<String>;
#[derive(Debug, ToSchema, Serialize, TS)]
#[ts(export, export_to = "../../../static_files/packages/shared/src/types/")]
pub struct WorkOrderSupervisorRow
{
    id: IdStringDto,
    area: Area,
    work_order_number: WorkOrderNumberDto,
    activity_number: ActivityNumber,
    permit: Permit,
    material: MaterialStatusDto,
    icc: Icc,
    description: Description,
    hands_on_tool_time: f64,
    hours_worked: f64,
    hours_planned: f64,
    percentage_complete: Percentage,
}

impl From<(&WorkOrder, ActivityNumber, IdStringDto)> for WorkOrderSupervisorRow
{
    fn from(value: (&WorkOrder, ActivityNumber, IdStringDto)) -> Self
    {
        // ISSUE #999 [ ] -
        let hands_on_tool_time = 0.0;
        let work_order_view = value.0.view();
        let hours_worked = 0.0;

        let hours_planned = work_order_view
            .operations
            .iter()
            .find(|e| e.activity == value.1)
            .expect("This the activity number should always be present")
            .remaining_work;

        let percentage = Percentage(hours_worked / hours_planned.to_f64());

        Self {
            id: value.2,
            area: None,
            work_order_number: WorkOrderNumberDto(value.0.work_order_number().0),
            activity_number: value.1,
            permit: None,
            material: work_order_view.material_status.into(),
            icc: None,
            description: work_order_view.description_work_order.clone(),
            // This is rather complicated. It dependes on the current state of the operational
            // Solution. You want to take the system solution and change this so that
            // the amount of work that is in the past is the hours worked. The
            // [`SystemClock`] will be crucial for the correct interpretation of this
            // value.
            hands_on_tool_time,
            hours_worked,
            hours_planned: hours_planned.to_f64(),
            percentage_complete: percentage,
        }
    }
}

#[derive(Debug, ToSchema, Serialize, TS)]
#[ts(export, export_to = "../../../static_files/packages/shared/src/types/")]
pub struct Percentage(f64);

// This should be a trait as well. You should not do it like this I believe
// This means that you will have to make a From implementation for each
// kind of SystemSolution. I am not that is the best.
impl TryFrom<(&WorkOrders, &TotalSystemSolution, &TimeEnvironment)> for SupervisorMainTableDto
{
    type Error = anyhow::Error;

    fn try_from(
        value: (&WorkOrders, &TotalSystemSolution, &TimeEnvironment),
    ) -> Result<Self, Self::Error>
    {
        let mut days = BTreeMap::default();

        let assigned_activities = &value
            .1
            .supervisor
            .as_ref()
            .unwrap()
            .assess_and_assign_activities();

        for day in value.2.frozen_days() {
            let mut work_order_activities_per_work_center: HashMap<
                String,
                Vec<WorkOrderSupervisorRow>,
            > = HashMap::default();

            for (id, work_order_activity) in assigned_activities {
                let operational_solutions = &value.1.operational;

                let operation_solution = operational_solutions.get(id).expect("Should");

                let operational_assignments_by_day = operation_solution
                    .scheduled_work_order_activities
                    // This value should be gotten from the
                    .iter()
                    .filter(|f| f.1.active_datetimes().contains(&day.date))
                    .find(|e| e.0 == *work_order_activity);

                if let Some(_operational_assignment) = operational_assignments_by_day {
                    let work_order_supervisor_row = WorkOrderSupervisorRow::from((
                        value
                            .0
                            .inner
                            .get(&work_order_activity.0)
                            .expect("WorkOrder should always be present here"),
                        work_order_activity.1,
                        IdStringDto::from(id.clone()),
                    ));

                    let resource = value
                        .0
                        .resource(work_order_activity)
                        .context("Could not find the work order and corresponding activity")?
                        .to_string();
                    work_order_activities_per_work_center
                        .entry(resource)
                        .or_default()
                        .push(work_order_supervisor_row);
                }
            }
            work_order_activities_per_work_center
                .iter_mut()
                .for_each(|e| e.1.sort_by(|a, b| a.id.cmp(&b.id)));
            let day_subtable = DaySubtable {
                work_order_activities_per_work_center,
            };
            days.insert(NaiveDateDto::from(day.date), day_subtable);
        }
        Ok(Self { days })
    }
}

// TODO [ ]
// Implement the other messages here as well
#[derive(Serialize, ToSchema)]
pub enum SupervisorResponseMessageDto
{
    Status(SupervisorResponseStatusDto),
    Scheduling,
    Resources,
    Time,
}

// CRUCIAL INSIGHT
// You are getting more mature here! You should simply keep it up.
#[derive(Serialize, ToSchema, TS)]
#[ts(export, export_to = "../../../static_files/packages/shared/src/types/")]
pub struct SupervisorResponseStatusDto
{
    pub delegated_work_order_activities: usize,
    pub objective: u64,
}

impl From<SupervisorResponseStatus> for SupervisorResponseStatusDto
{
    fn from(value: SupervisorResponseStatus) -> Self
    {
        Self {
            delegated_work_order_activities: value.delegated_work_order_activities,
            objective: value.objective,
        }
    }
}
impl From<SupervisorResponseMessage> for SupervisorResponseMessageDto
{
    fn from(value: SupervisorResponseMessage) -> Self
    {
        match value {
            SupervisorResponseMessage::StateLink => todo!(),
            SupervisorResponseMessage::Status(supervisor_response_status) => {
                Self::Status(supervisor_response_status.into())
            }
            SupervisorResponseMessage::Scheduling(_supervisor_response_scheduling) => todo!(),
            SupervisorResponseMessage::Resources(_supervisor_response_resources) => todo!(),
            SupervisorResponseMessage::Time(_supervisor_response_time) => todo!(),
        }
    }
}
// I think that you should take a break now. The most important thing here is to
// make sure that the system runs as smoothly as possible. The frontend should
// not decide a single thing here.

// This can be derived uniquely from the SystemSolution
impl From<SupervisorSolution> for SupervisorResourcesDto
{
    fn from(value: SupervisorSolution) -> Self
    {
        let all_technicians = value.all_technicians();
        let assigned_activities = value.assigned_activities();
        SupervisorResourcesDto {
            assigned_activities: assigned_activities
                .into_iter()
                .map(|e| (IdStringDto::from(e.0), WorkOrderActivityDto::from(e.1)))
                .collect(),
            all_technicians: all_technicians.into_iter().map(IdDto::from).collect(),
        }
    }
}

#[derive(Debug, Serialize, ToSchema, TS)]
#[ts(export, export_to = "../../../static_files/packages/shared/src/types/")]
pub struct TechnicianAvailability
{
    pub id: IdStringDto,
    pub resources: Vec<String>,
    pub start: NaiveDateDto,
    pub end: NaiveDateDto,
}

#[derive(ToSchema, Serialize, Debug, TS)]
#[ts(export, export_to = "../../../static_files/packages/shared/src/types/")]
pub struct SupervisorAllAvailableTechnicians
{
    all_technicians: Vec<TechnicianAvailability>,
}
impl From<BTreeMap<IdString, (BTreeSet<Availability>, HashSet<Resources>)>>
    for SupervisorAllAvailableTechnicians
{
    fn from(
        value: BTreeMap<String, (std::collections::BTreeSet<Availability>, HashSet<Resources>)>,
    ) -> Self
    {
        let all_technicians = value
            .iter()
            .flat_map(|(id, e)| {
                e.0.iter().map(|availability| TechnicianAvailability {
                    id: IdStringDto::new(id.clone()),
                    resources: e.1.iter().map(|e| e.to_string()).collect::<Vec<_>>(),
                    start: NaiveDateDto(
                        availability.start_datetime().format("%Y-%m-%d").to_string(),
                    ),
                    end: NaiveDateDto(
                        availability
                            .finish_datetime()
                            .format("%Y-%m-%d")
                            .to_string(),
                    ),
                })
            })
            .collect();

        Self { all_technicians }
    }
}
