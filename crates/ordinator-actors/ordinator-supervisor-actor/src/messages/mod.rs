pub mod message_handlers;
pub mod requests;
pub mod responses;

use ordinator_actor_core::RequestMessage;
use requests::SupervisorRequestResource;
use requests::SupervisorRequestScheduling;
use requests::SupervisorSchedulingEnvironmentCommands;
use requests::SupervisorStatusMessage;
use requests::SupervisorTimeRequest;
use responses::SupervisorResponseResources;
use responses::SupervisorResponseScheduling;
use responses::SupervisorResponseStatus;
use responses::SupervisorResponseTime;
use serde::Deserialize;
use serde::Serialize;

pub type SupervisorRequestMessage = RequestMessage<
    SupervisorStatusMessage,
    SupervisorRequestScheduling,
    SupervisorRequestResource,
    SupervisorTimeRequest,
    SupervisorSchedulingEnvironmentCommands,
>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SupervisorType
{
    Main,
    Other,
}

// You cannot have this one here. You have to do something else instead. I think
// that the best approach is to make a `From` implementation.
//
#[derive(Debug, Serialize)]
pub enum SupervisorResponseMessage
{
    StateLink,
    Status(SupervisorResponseStatus),
    Scheduling(SupervisorResponseScheduling),
    Resources(SupervisorResponseResources),
    Time(SupervisorResponseTime),
    // Test(AlgorithmState<SupervisorInfeasibleCases>),
}

impl SupervisorResponseMessage
{
    pub fn status(self) -> SupervisorResponseStatus
    {
        match self {
            Self::Status(supervisor_response_status) => supervisor_response_status,
            _ => panic!("The underlying variant of the enum was not a status response"),
        }
    }
}

// #[derive(Serialize)]
// pub struct SupervisorInfeasibleCases {
//     pub respect_main_work_center: ConstraintState<String>,
// }

// impl Default for SupervisorInfeasibleCases {
//     fn default() -> Self {
//         Self {
//             respect_main_work_center:
// ConstraintState::Infeasible("Infeasible".to_string()),         }
//     }
// }
