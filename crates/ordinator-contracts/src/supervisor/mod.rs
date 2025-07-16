use std::collections::BTreeMap;
use std::collections::BTreeSet;

use ordinator_supervisor_actor::algorithm::supervisor_solution::SupervisorSolution;
use ordinator_supervisor_actor::messages::SupervisorResponseMessage;
use ordinator_supervisor_actor::messages::responses::SupervisorResponseStatus;
use serde::Serialize;
use utoipa::ToSchema;

use crate::IdDto;
use crate::WorkOrderActivityDto;

#[derive(ToSchema, Serialize)]
pub struct SupervisorResources
{
    all_technicians: BTreeSet<IdDto>,
    assigned_activities: BTreeMap<IdDto, WorkOrderActivityDto>,
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
#[derive(Serialize, ToSchema)]
pub struct SupervisorResponseStatusDto
{
    pub supervisor_resource: Vec<IdDto>,
    pub delegated_work_order_activities: usize,
    pub objective: u64,
}

impl From<SupervisorResponseStatus> for SupervisorResponseStatusDto
{
    fn from(value: SupervisorResponseStatus) -> Self
    {
        Self {
            supervisor_resource: value
                .supervisor_resource
                .into_iter()
                .map(IdDto::from)
                .collect(),
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
impl From<SupervisorSolution> for SupervisorResources
{
    fn from(value: SupervisorSolution) -> Self
    {
        let all_technicians = value.all_technicians();
        let assigned_activities = value.assigned_activities();
        SupervisorResources {
            assigned_activities: assigned_activities
                .into_iter()
                .map(|e| (IdDto::from(e.0), WorkOrderActivityDto::from(e.1)))
                .collect(),
            all_technicians: all_technicians.into_iter().map(IdDto::from).collect(),
        }
    }
}
