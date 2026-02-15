pub mod message_handlers;
pub mod requests;
pub mod responses;
use ordinator_actor_core::RequestMessage;
use ordinator_scheduling_environment::worker_environment::resources::ActorCompositeId;
use serde::Serialize;

use self::requests::*;
use self::responses::*;

pub type OperationalRequestMessage = RequestMessage<
    OperationalStatusRequest,
    OperationalSchedulingRequest,
    OperationalResourceRequest,
    OperationalTimeRequest,
    OperationalSchedulingEnvironmentCommands,
>;

// Type-safe response message variant for handling multiple response types
pub enum ResponseMessage<S, Sc, R, T>
{
    Status(S),
    Scheduling(Sc),
    Resource(R),
    Time(T),
}

// Use module paths for namespacing (e.g., `operational::response::Status`, `supervisor::request::Status`)
#[derive(Debug, Serialize)]
pub enum OperationalResponseMessage
{
    Status(OperationalResponseStatus),
    Scheduling(OperationalSchedulingResponse),
    Resource(OperationalResourceResponse),
    Time(OperationalTimeResponse),
    Success,
}

#[derive(Serialize)]
pub struct OperationalStatus
{
    objective: f64,
}

// Wrapper enum for different operational response types
#[derive(Serialize)]
pub enum OperationalResponse
{
    AllOperationalStatus(Vec<OperationalResponseMessage>),
    OperationalIds(Vec<ActorCompositeId>),
    OperationalState(OperationalResponseMessage),
    NoOperationalAgentFound(String),
}

// #[derive(Clone, Deserialize, Serialize, Debug, clap::ValueEnum)]
// pub enum OperationalTarget {
//     #[clap(skip)]
//     Single(OperationalId),
//     All,
// }

// #[derive(Serialize)]
// pub struct OperationalInfeasibleCases {
//     pub operation_overlap: ConstraintState<String>,
// }

// impl OperationalInfeasibleCases {
//     pub fn all_feasible(&self) -> bool {
//         if self.operation_overlap != ConstraintState::Feasible {
//             return false;
//         }
//         true
//     }
// }

// impl Default for OperationalInfeasibleCases {
//     fn default() -> Self {
//         Self {
//             operation_overlap: ConstraintState::Undetermined,
//         }
//     }
// }

#[cfg(test)]
mod tests
{

    use chrono::DateTime;
    use chrono::NaiveTime;
    use chrono::TimeDelta;
    use ordinator_scheduling_environment::time_environment::TimeInterval;

    #[test]
    fn test_time_interval_contains_1()
    {
        let start_time = NaiveTime::from_hms_opt(19, 00, 00).unwrap();
        let end_time = NaiveTime::from_hms_opt(1, 0, 0).unwrap();

        let current_time = DateTime::parse_from_rfc3339("2024-07-14T00:00:00Z")
            .unwrap()
            .to_utc();

        let time_interval = TimeInterval::new(start_time, end_time).unwrap();

        assert!(time_interval.contains(&current_time));
    }
    #[test]
    fn test_time_interval_contains_2()
    {
        let start_time = NaiveTime::from_hms_opt(19, 00, 00).unwrap();
        let end_time = NaiveTime::from_hms_opt(22, 0, 0).unwrap();

        let current_time = DateTime::parse_from_rfc3339("2024-07-14T20:00:00Z")
            .unwrap()
            .to_utc();

        let time_interval = TimeInterval::new(start_time, end_time).unwrap();

        assert!(time_interval.contains(&current_time));
    }

    #[test]
    fn test_time_interval_contains_3()
    {
        let start_time = NaiveTime::from_hms_opt(19, 00, 00).unwrap();
        let end_time = NaiveTime::from_hms_opt(1, 0, 0).unwrap();

        let current_time = DateTime::parse_from_rfc3339("2024-07-14T18:00:00Z")
            .unwrap()
            .to_utc();

        let time_interval = TimeInterval::new(start_time, end_time).unwrap();

        assert!(!time_interval.contains(&current_time));
    }
    #[test]
    fn test_time_interval_contains_4()
    {
        let start_time = NaiveTime::from_hms_opt(19, 00, 00).unwrap();
        let end_time = NaiveTime::from_hms_opt(22, 0, 0).unwrap();

        let current_time = DateTime::parse_from_rfc3339("2024-07-14T18:00:00Z")
            .unwrap()
            .to_utc();

        let time_interval = TimeInterval::new(start_time, end_time).unwrap();

        assert!(!time_interval.contains(&current_time));
    }

    #[test]
    fn test_time_interval_duration()
    {
        let start = NaiveTime::from_hms_opt(19, 00, 00).unwrap();
        let end = NaiveTime::from_hms_opt(7, 00, 00).unwrap();
        let time_interval = TimeInterval { start, end };
        assert_eq!(
            time_interval.duration(),
            TimeDelta::new(12 * 3600, 0).unwrap()
        );

        let start = NaiveTime::from_hms_opt(2, 00, 00).unwrap();
        let end = NaiveTime::from_hms_opt(7, 00, 00).unwrap();
        let time_interval = TimeInterval { start, end };
        assert_eq!(
            time_interval.duration(),
            TimeDelta::new(5 * 3600, 0).unwrap()
        );
        let start = NaiveTime::from_hms_opt(23, 00, 00).unwrap();
        let end = NaiveTime::from_hms_opt(1, 00, 00).unwrap();
        let time_interval = TimeInterval { start, end };
        assert_eq!(
            time_interval.duration(),
            TimeDelta::new(2 * 3600, 0).unwrap()
        );

        let start = NaiveTime::from_hms_opt(23, 00, 00).unwrap();
        let end = NaiveTime::from_hms_opt(23, 00, 00).unwrap();
        let time_interval = TimeInterval { start, end };
        assert_eq!(time_interval.duration(), TimeDelta::new(0, 0).unwrap());
    }
}
