pub mod message_handlers;
pub mod requests;
pub mod responses;

use ordinator_actor_core::RequestMessage;
use requests::ProjectRequestResource;
use requests::ProjectRequestScheduling;
use requests::ProjectSchedulingEnvironmentCommands;
use requests::ProjectStatusMessage;
use requests::ProjectTimeRequest;
use responses::ProjectResponseScheduling;
use responses::ProjectResponseStatus;
use responses::ProjectResponseTime;
use serde::Serialize;

pub type ProjectRequestMessage = RequestMessage<
    ProjectStatusMessage,
    ProjectRequestScheduling,
    ProjectRequestResource,
    ProjectTimeRequest,
    ProjectSchedulingEnvironmentCommands,
>;

#[derive(Debug, Serialize)]
pub enum ProjectResponseMessage
{
    FreeStringResponse(String),
    Status(ProjectResponseStatus),
    Scheduling(ProjectResponseScheduling),
    // Resources(ProjectResourceResponse),
    Time(ProjectResponseTime),
    Update,
}

// TODO: Consider reintroducing this code at a later stage; the idea is sound.
// #[derive(Debug, Clone, Serialize)]
// pub struct ProjectInfeasibleCases {
//     pub aggregated_load: ConstraintState<String>,
//     pub earliest_start_day: ConstraintState<String>,
//     pub all_scheduled: ConstraintState<String>,
//     pub respect_period_id: ConstraintState<String>,
// }

// impl Default for ProjectInfeasibleCases {
//     fn default() -> Self {
//         ProjectInfeasibleCases {
//             aggregated_load:
// ConstraintState::Infeasible("Infeasible".to_owned()),
// earliest_start_day: ConstraintState::Infeasible("Infeasible".to_owned()),
//             all_scheduled:
// ConstraintState::Infeasible("Infeasible".to_owned()),
// respect_period_id: ConstraintState::Infeasible("Infeasible".to_owned()),
//         }
//     }
// }
