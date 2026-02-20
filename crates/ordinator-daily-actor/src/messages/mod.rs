pub mod message_handlers;
pub mod requests;
pub mod responses;

use ordinator_actor_core::RequestMessage;
use requests::DailyRequestResource;
use requests::DailyRequestScheduling;
use requests::DailySchedulingEnvironmentCommands;
use requests::DailyStatusMessage;
use requests::DailyTimeRequest;
use responses::DailyResponseResources;
use responses::DailyResponseScheduling;
use responses::DailyResponseStatus;
use responses::DailyResponseTime;
use serde::Deserialize;
use serde::Serialize;

pub type DailyRequestMessage = RequestMessage<
    DailyStatusMessage,
    DailyRequestScheduling,
    DailyRequestResource,
    DailyTimeRequest,
    DailySchedulingEnvironmentCommands,
>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum DailyType
{
    Main,
    Other,
}

// Implement `From` trait instead of implementing this directly
#[derive(Debug, Serialize)]
pub enum DailyResponseMessage
{
    StateLink,
    Status(DailyResponseStatus),
    Scheduling(DailyResponseScheduling),
    Resources(DailyResponseResources),
    Time(DailyResponseTime),
    // Test(AlgorithmState<DailyInfeasibleCases>),
}

impl DailyResponseMessage
{
    pub fn status(self) -> DailyResponseStatus
    {
        match self {
            Self::Status(daily_response_status) => daily_response_status,
            _ => panic!("The underlying variant of the enum was not a status response"),
        }
    }
}

// #[derive(Serialize)]
// pub struct DailyInfeasibleCases {
//     pub respect_main_work_center: ConstraintState<String>,
// }

// impl Default for DailyInfeasibleCases {
//     fn default() -> Self {
//         Self {
//             respect_main_work_center:
// ConstraintState::Infeasible("Infeasible".to_string()),         }
//     }
// }
