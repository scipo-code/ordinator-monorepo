use std::collections::HashMap;
use std::collections::HashSet;

use anyhow::Result;
use anyhow::ensure;
use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

use crate::time_environment::day::Day;
use crate::time_environment::period::Period;
use crate::work_order;
use crate::work_order::ForcedWorkOrder;
use crate::work_order::WorkOrderNumber;
use crate::work_order::operation::ActivityNumber;
use crate::worker_environment::IdString;

pub type AssignmentId = Uuid;

// Uses TypeState pattern to ensure the system works with correct data.
// SavedAssignments are accessed as a repository through traits, with each
// Assignment referenced by ID and containing WorkOrderNumber and ActivityNumber.
#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct SavedAssignment
{
    assignments: HashMap<AssignmentId, AnyAssignment>,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub enum AnyAssignment
{
    Base(Assignment<BaseAssignment>),
}
impl AnyAssignment
{
    pub fn work_order_number(&self) -> WorkOrderNumber
    {
        match self {
            AnyAssignment::Base(assignment) => assignment.work_order_number,
        }
    }

    pub fn day(&self) -> Option<Day>
    {
        match self {
            AnyAssignment::Base(assignment) => assignment.day.clone(),
        }
    }

    pub fn activity_number(&self) -> Option<ActivityNumber>
    {
        match self {
            AnyAssignment::Base(assignment) => assignment.activity_number,
        }
    }
}

impl SavedAssignment
{
    // Creates an assignment for the given technicians with the specified work order.
    // SavedAssignments are always dependent on the technician to enable assigning
    // a [`WorkOrderActivity`] to a single technician.
    pub fn make_assignment_for_technician(
        &mut self,
        work_order_number: WorkOrderNumber,
        work_order: &ForcedWorkOrder,
        activity_number: &ActivityNumber,
        id: &[IdString],
    ) -> Result<()>
    {
        // TODO: Create correct structure to hold data. Uses WorkOrder to overwrite
        // SavedAssignments entries. Note: Code is overwritten on each change rather
        // than modified; proceed with implementation and handle edge cases iteratively.
        let assignment = match work_order {
            ForcedWorkOrder::Period(period) => {
                let technicians = id.iter().map(|e| (e.clone(), None)).collect::<HashSet<_>>();
                Assignment::new(
                    work_order_number,
                    Some(*activity_number),
                    Some(period.0.clone()),
                    None,
                    technicians,
                )
            }
            ForcedWorkOrder::Days(project_force_type) => match project_force_type {
                work_order::ProjectForceType::OnlyStartDay(day) => {
                    let technicians = id.iter().map(|e| (e.clone(), None)).collect::<HashSet<_>>();
                    Assignment::new(
                        work_order_number,
                        Some(*activity_number),
                        None,
                        Some(day.clone()),
                        technicians,
                    )
                }
                work_order::ProjectForceType::IndividualActivities(_vec, _vec1) => todo!(),
            },
            ForcedWorkOrder::Technician(technician_include, _technician_exclude) => {
                let date_time_option = technician_include.interval.as_ref().map(|day| day.0);
                let technicians = id
                    .iter()
                    // Note: Modifying Technicians is nearly impossible as ID is large/complex.
                    .map(|e| (e.clone(), date_time_option))
                    .collect::<HashSet<_>>();
                Assignment::new(work_order_number, None, None, None, technicians)
            }
            ForcedWorkOrder::FreeWorkOrder => {
                let technicians = id.iter().map(|e| (e.clone(), None)).collect::<HashSet<_>>();
                Assignment::new(work_order_number, None, None, None, technicians)
            }
        };

        // Forced work orders take precedence over other constraints.
        let assignment = AnyAssignment::Base(assignment);
        self.assignments.insert(Uuid::new_v4(), assignment);

        Ok(())
    }

    pub fn make_assignment_for_project(
        &mut self,
        work_order_number: WorkOrderNumber,
        work_order: &ForcedWorkOrder,
        day: Day,
    ) -> Result<()>
    {
        // Creates a project assignment with the given WorkOrder and day. Returns
        // an error if the day conflicts with the work order constraints.
        let assignment = match work_order {
            ForcedWorkOrder::Period(period) => {
                ensure!(
                    period.0.contains_date(day.date),
                    "WorkOrder is scheduled for period {:#?}, assigning basic start for {:#?} is not allowed",
                    period.0,
                    day
                );

                Assignment::new(
                    work_order_number,
                    None,
                    Some(period.0.clone()),
                    Some(day),
                    HashSet::new(),
                )
            }
            ForcedWorkOrder::Days(project_force_type) => match project_force_type {
                work_order::ProjectForceType::OnlyStartDay(work_order_day) => {
                    ensure!(
                        *work_order_day == day,
                        "WorkOrder is scheduled for day {:#?}, assigning basic start for {:#?} is not allowed",
                        work_order_day,
                        day,
                    );
                    Assignment::new(
                        work_order_number,
                        None,
                        None,
                        Some(day.clone()),
                        HashSet::new(),
                    )
                }
                work_order::ProjectForceType::IndividualActivities(_vec, _vec1) => todo!(),
            },
            // TODO: Handle plural technicians.
            ForcedWorkOrder::Technician(technician_include, _technician_exclude) => {
                let technicians = technician_include.id.clone();
                let hash_set = HashSet::from([(technicians, None)]);
                Assignment::new(work_order_number, None, None, Some(day), hash_set)
            }
            ForcedWorkOrder::FreeWorkOrder => {
                Assignment::new(work_order_number, None, None, Some(day), HashSet::new())
            }
        };

        // Forced work orders take precedence over other constraints.
        let assignment = AnyAssignment::Base(assignment);
        self.assignments.insert(Uuid::new_v4(), assignment);

        Ok(())
    }

    pub fn assignment_for_project(&self) -> Vec<(&Uuid, &AnyAssignment)>
    {
        self.assignments
            .iter()
            .filter(|&e| match e.1 {
                AnyAssignment::Base(assignment) => assignment.day.is_some(),
            })
            .collect::<Vec<_>>()
    }

    pub(crate) fn new(assignments: HashMap<Uuid, AnyAssignment>) -> Self
    {
        Self { assignments }
    }
}

// Assignment is a value object combining time value objects, IDs, and work order
// information to ensure correct state handling. Uses TypeState to manage different
// assignment states. The WorkOrder API is clean: creating assignments takes a
// WorkOrder instance and returns a Result. WorkOrder mutations are handled separately.
#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct Assignment<State>
{
    work_order_number: WorkOrderNumber,
    activity_number: Option<ActivityNumber>,
    period: Option<Period>,
    day: Option<Day>,
    technician: HashSet<(IdString, Option<DateTime<Utc>>)>,
    state: State,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct BaseAssignment;

impl Assignment<BaseAssignment>
{
    pub fn new(
        work_order_number: WorkOrderNumber,
        activity_number: Option<ActivityNumber>,
        period: Option<Period>,
        day: Option<Day>,
        technician: HashSet<(IdString, Option<DateTime<Utc>>)>,
    ) -> Self
    {
        let state = BaseAssignment;
        Self {
            period,
            day,
            technician,
            state,
            work_order_number,
            activity_number,
        }
    }
}

impl<State> Assignment<State>
{
    pub fn day(&self) -> Option<Day>
    {
        self.day.clone()
    }
}
