use ordinator_actor_operational::algorithm::operational_solution::OperationalAssignment;
use ordinator_scheduling_environment::work_order::operation::ActivityNumber;
use ordinator_scheduling_environment::work_order::WorkOrderNumber;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(PartialEq, Eq, PartialOrd, Ord, ToSchema, Serialize)]
pub struct OperationalAssignmentsDto
{
    work_order_activity: (u64, ActivityNumber),
    start: String,
    finish: String,
}

impl From<((WorkOrderNumber, ActivityNumber), OperationalAssignment)> for OperationalAssignmentsDto
{
    fn from(value: ((WorkOrderNumber, ActivityNumber), OperationalAssignment)) -> Self
    {
        // TODO: Verify and handle edge cases in assignment conversion
        Self {
            work_order_activity: (value.0 .0 .0, value.0 .1),
            start: value.1.start_time().to_rfc3339(),
            finish: value.1.finish_time().to_rfc3339(),
        }
    }
}
