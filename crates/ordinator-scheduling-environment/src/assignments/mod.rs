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

// ESSAY: #001
// I think that I should make the `TypeState` pattern here. Yes make the
// TypeState pattern on the `Assignment` and use this to make sure that the
// system works on the correct data. Should the SavedAssignments be modeled
// this way? No I do not think that is a good idea.
//     `SavedAssignment`s are a repository, you should interact with it through
// a trait later on. I think that the best approach here is to refer to each
// of the `Assignment`s through an Id and then move the `WorkOrderNumber` and
// `Activity` number into the the `Assignment`
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

// Now you have to determine what it is that you want to force here. I think
// that the best approach is to make the system function with the correct
impl SavedAssignment
{
    // This should be extended to handle the correct initialization of the system.
    // The best approach is to make the `SavedAssignment`s always dependent on
    // the technician.
    // WHY:
    // To make it possible to assign a [`WorkOrderActivity`] to a single technician.
    pub fn make_assignment_for_technician(
        &mut self,
        work_order_number: WorkOrderNumber,
        work_order: &ForcedWorkOrder,
        activity_number: &ActivityNumber,
        id: &[IdString],
    ) -> Result<()>
    {
        // TODO [ ] - you have to create the correct structure to hold the data.
        //
        // Yes this is the way! You are now using the `WorkOrder` has a mechanism to
        // overwrite the what ever is in the `SavedAssignments`.
        //
        // This function is completely wrong, the issue is that the code is not modified
        // but over written on each change. That is not the intended behavior.
        // As long as it works you should keep moving forward. I do not see them
        // make a test that fails the code. Yes, do not guess, simply keep implementing
        // and then work on the edge cases and observe the hidden abstraction
        // afterwards.
        let assignment = match work_order {
            ForcedWorkOrder::Period(period) => {
                let technicians = id.iter().map(|e| (e.clone(), None)).collect::<HashSet<_>>();
                // Should we make an assignment for each of the technicians? Yes.
                Assignment::new(
                    work_order_number,
                    Some(*activity_number),
                    Some(period.0.clone()),
                    None,
                    technicians,
                )
            }
            ForcedWorkOrder::Days(tactical_force_type) => match tactical_force_type {
                work_order::TacticalForceType::OnlyStartDay(day) => {
                    let technicians = id.iter().map(|e| (e.clone(), None)).collect::<HashSet<_>>();
                    Assignment::new(
                        work_order_number,
                        Some(*activity_number),
                        None,
                        Some(day.clone()),
                        technicians,
                    )
                }
                work_order::TacticalForceType::IndividualActivities(_vec, _vec1) => todo!(),
            },
            ForcedWorkOrder::Technician(technician_include, _technician_exclude) => {
                let date_time_option = technician_include.interval.as_ref().map(|day| day.0);
                let technicians = id
                    .iter()
                    // WARN [ ] modifying Technicians in almost impossible as ID is FAT.
                    .map(|e| (e.clone(), date_time_option))
                    .collect::<HashSet<_>>();
                Assignment::new(work_order_number, None, None, None, technicians)
            }
            ForcedWorkOrder::FreeWorkOrder => {
                let technicians = id.iter().map(|e| (e.clone(), None)).collect::<HashSet<_>>();
                Assignment::new(work_order_number, None, None, None, technicians)
            }
        };

        // The forced work order should take precedence,
        let assignment = AnyAssignment::Base(assignment);
        self.assignments.insert(Uuid::new_v4(), assignment);

        Ok(())
    }

    pub fn make_assignment_for_tactical(
        &mut self,
        work_order_number: WorkOrderNumber,
        work_order: &ForcedWorkOrder,
        day: Day,
    ) -> Result<()>
    {
        // There is a difference between the WorkOrder and the other parts of the
        // program. Should you stop? This function should change the Tactical
        // day, unless it is being blocked by something else. It should return
        // error codes.
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
            ForcedWorkOrder::Days(tactical_force_type) => match tactical_force_type {
                work_order::TacticalForceType::OnlyStartDay(work_order_day) => {
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
                work_order::TacticalForceType::IndividualActivities(_vec, _vec1) => todo!(),
            },
            // TODO [ ] The technician can be plural.
            ForcedWorkOrder::Technician(technician_include, _technician_exclude) => {
                let technicians = technician_include.id.clone();
                let hash_set = HashSet::from([(technicians, None)]);
                Assignment::new(work_order_number, None, None, Some(day), hash_set)
            }
            ForcedWorkOrder::FreeWorkOrder => {
                Assignment::new(work_order_number, None, None, Some(day), HashSet::new())
            }
        };

        // The forced work order should take precedence,
        let assignment = AnyAssignment::Base(assignment);
        self.assignments.insert(Uuid::new_v4(), assignment);

        Ok(())
    }

    pub fn assignment_for_tactical(&self) -> Vec<(&Uuid, &AnyAssignment)>
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

// REMEMBER:
// * Exclude
// * Correct data structure
// * Keep it simple
// This is everything I believe.
//
// ESSAY: #002
// Should the Assignments be a value object? I think that it
// should be. It should use a combination of time value objects
// ids and other parts of the system to correctly create an
// assignment.
//     So you should make sure that the correct states are handled here.
// All the different states should be expressed and handled in here centrally
// you do not know what to do now and that means that you should make a couple
// of use cases and see what you learn. What should you do now then?
// * Clean API on the WorkOrder
// * Making an assignment should take a `WorkOrder` instance.
// * The `WorkOrder` instance should be used to return a `Result` with further
//   actions
// * The `WorkOrder` object cannot be mutated here. That is a separate endpoint.
//   This is a good
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
