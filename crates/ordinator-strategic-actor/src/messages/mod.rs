pub mod message_handlers;
pub mod requests;
pub mod responses;

use ordinator_actor_core::RequestMessage;
use ordinator_scheduling_environment::work_order::work_order_analytic::status_codes::WeeklyUserStatusCodes;
use serde::Deserialize;
use serde::Serialize;

use self::requests::*;
use self::responses::*;

pub type WeeklyRequestMessage = RequestMessage<
    WeeklyStatusMessage,
    WeeklyRequestScheduling,
    WeeklyRequestResource,
    WeeklyTimeRequest,
    WeeklySchedulingEnvironmentCommands,
>;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum WeeklySchedulingEnvironmentCommands
{
    UserStatus(WeeklyUserStatusCodes),
}

#[derive(Serialize)]
#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum WeeklyResponseMessage
{
    StateLink,
    Status(WeeklyResponseStatus),
    Scheduling(WeeklyResponseScheduling),
    Resources(WeeklyResponseResources),
    Periods(WeeklyResponsePeriods),
    Success,
}

// #[derive(Serialize)]
// pub struct WeeklyInfeasibleCases {
//     pub respect_awsc: ConstraintState<String>,
//     pub respect_unloading: ConstraintState<String>,
//     pub respect_sch: ConstraintState<String>,
//     pub respect_aggregated_load: ConstraintState<String>,
// }

// impl Default for WeeklyInfeasibleCases {
//     fn default() -> Self {
//         WeeklyInfeasibleCases {
//             respect_awsc:
// ConstraintState::Infeasible("Infeasible".to_string()),
// respect_unloading: ConstraintState::Infeasible("Infeasible".to_string()),
//             respect_sch:
// ConstraintState::Infeasible("Infeasible".to_string()),
// respect_aggregated_load:
// ConstraintState::Infeasible("Infeasible".to_string()),         }
//     }
// }
