use std::collections::HashMap;
use std::ops::ControlFlow;

// The correct implementation depends on runtime program state that is unknown at this point
use anyhow::Context;
use anyhow::Result;
use anyhow::ensure;
use chrono::DateTime;
use chrono::NaiveDate;
use chrono::Utc;
use colored::Colorize;
use ordinator_actor_core::traits::ObjectiveValue;
use ordinator_orchestrator_actor_traits::Solution;
use ordinator_orchestrator_actor_traits::SolutionState;
use ordinator_orchestrator_actor_traits::SwapSolution;
use ordinator_orchestrator_actor_traits::SystemSolutions;
use ordinator_orchestrator_actor_traits::marginal_fitness::MarginalFitness;
use ordinator_scheduling_environment::time_environment::TimeInterval;
use ordinator_scheduling_environment::time_environment::day::Day;
use ordinator_scheduling_environment::work_order::ActivityRelation;
use ordinator_scheduling_environment::work_order::WorkOrderActivity;
use ordinator_scheduling_environment::work_order::WorkOrderNumber;
use ordinator_scheduling_environment::worker_environment::availability::Availability;
use ordinator_scheduling_environment::worker_environment::resources::ActorCompositeId;
use serde::Serialize;
use valuable::Valuable;

// Used by contracts, conversions, and the orchestrator
use super::ContainOrNextOrNone;
use super::Unavailability;
use super::no_overlap_by_ref;
use super::operational_events::OperationalEvents;
use super::operational_parameter::OperationalParameters;

/// Objective value for operational actor optimization, enabling implementation of traits and custom formatting
#[derive(Serialize, Copy, PartialEq, PartialOrd, Ord, Eq, Debug, Default, Clone, Valuable)]
pub struct OperationalObjectiveValue
{
    /// Hands-on tool time utilization in milliseconds
    hands_on_tool_time: u64,
    /// Assessment activity count
    assess: u64,
    /// Assignment activity count
    assign: u64,
    /// Total scheduled work order activities
    total_work_order_activities: u64,
}

impl ObjectiveValue for OperationalObjectiveValue {}

impl From<(u64, u64, u64, u64)> for OperationalObjectiveValue
{
    fn from(value: (u64, u64, u64, u64)) -> Self
    {
        Self {
            hands_on_tool_time: value.0,
            assess: value.1,
            assign: value.2,
            total_work_order_activities: value.3,
        }
    }
}

#[derive(PartialEq, Eq, Default, Clone)]
pub struct OperationalSolution
{
    pub(crate) objective_value: OperationalObjectiveValue,
    pub(crate) scheduled_work_order_activities: Vec<(WorkOrderActivity, OperationalAssignment)>,
    pub(crate) non_productive: Vec<Assignment>,
}

// Debug implementation depends on formatting flags and anticipates future requirements
impl Debug for OperationalSolution
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        if f.alternate() {
            write!(
                f,
                "OperationalSolution\
                {{\
                objective_value: {:#?}\n\
                scheduled_activities: {}\n\
                }}",
                self.objective_value,
                self.scheduled_work_order_activities.len(),
            )
        } else {
            Ok(())
        }
    }
}
use std::fmt::Debug;

impl Solution for OperationalSolution
{
    type Objective = OperationalObjectiveValue;
    type Parameters = OperationalParameters;

    fn from_parameters(parameters: &Self::Parameters) -> Result<Self>
    {
        let mut scheduled_work_order_activities = Vec::new();

        let start_event =
            Assignment::make_unavailable_event(Unavailability::Beginning, &parameters.availability)
                .context("Could not make unavailability event for the OperationalActor")?;

        let end_event =
            Assignment::make_unavailable_event(Unavailability::End, &parameters.availability)
                .context("Could not make unavailability event for the OperationalActor")?;

        let unavailability_start_event = OperationalAssignment::new(vec![start_event]);

        let unavailability_end_event = OperationalAssignment::new(vec![end_event]);

        scheduled_work_order_activities.push(((WorkOrderNumber(0), 0), unavailability_start_event));

        scheduled_work_order_activities.push(((WorkOrderNumber(0), 0), unavailability_end_event));

        Ok(Self {
            objective_value: OperationalObjectiveValue {
                hands_on_tool_time: 0,
                assess: 0,
                assign: 0,
                total_work_order_activities: 0,
            },
            scheduled_work_order_activities,
            non_productive: vec![],
        })
    }

    fn update_objective(&mut self, other_objective_value: Self::Objective)
    {
        self.objective_value = other_objective_value;
    }
}

// Implement SwapSolution trait for generic system-wide swap operations
impl<Ss> SwapSolution<Ss> for OperationalSolution
where
    Ss: SystemSolutions<Operational = Self>,
{
    fn swap(id: &ActorCompositeId, solution: SolutionState<Self>, system_solution: &mut Ss)
    {
        system_solution.operational_swap(id, solution);
    }
}

#[allow(dead_code)]
pub trait GetMarginalFitness
{
    fn marginal_fitness(
        &self,
        operational_agent: &ActorCompositeId,
        work_order_activity: &WorkOrderActivity,
    ) -> Result<&MarginalFitness>;
}
impl GetMarginalFitness for HashMap<ActorCompositeId, OperationalSolution>
{
    fn marginal_fitness(
        &self,
        operational_agent: &ActorCompositeId,
        work_order_activity: &WorkOrderActivity,
    ) -> Result<&MarginalFitness>
    {
        self.get(operational_agent)
            .with_context(|| {
                format!(
                    "Could not find {} for operational agent: {:#?}",
                    std::any::type_name::<MarginalFitness>(),
                    operational_agent,
                )
            })?
            .scheduled_work_order_activities
            .iter()
            .find(|woa_os| woa_os.0 == *work_order_activity)
            .map(|os| &os.1.marginal_fitness)
            .with_context(|| {
                format!(
                    "{} did not have\n{:#?}",
                    operational_agent.to_string().bright_blue(),
                    format!("{work_order_activity:#?}",).bright_yellow()
                )
            })
    }
}

/// These are methods for the public API of the `OperationalSolution`.
impl OperationalSolution
{
    pub fn is_operational_solution_already_scheduled(
        &self,
        work_order_activity: WorkOrderActivity,
    ) -> bool
    {
        self.scheduled_work_order_activities
            .iter()
            .any(|(woa, _)| *woa == work_order_activity)
    }

    pub fn operational_assignments_by_day(
        &self,
        work_order_activity: &WorkOrderActivity,
        day: &Day,
    ) -> Option<&((WorkOrderNumber, u64), OperationalAssignment)>
    {
        self.scheduled_work_order_activities
            .iter()
            .filter(|f| f.1.active_datetimes().contains(&day.date))
            .find(|e| e.0 == *work_order_activity)
    }

    pub fn all_scheduled_work_order_activities(
        &self,
    ) -> Vec<((WorkOrderNumber, u64), OperationalAssignment)>
    {
        self.scheduled_work_order_activities.clone()
    }
}

impl OperationalSolution
{
    pub(super) fn try_insert(
        &mut self,
        work_order_activity: WorkOrderActivity,
        assignments: Vec<Assignment>,
        _activity_relation: ActivityRelation,
    ) -> Option<WorkOrderActivity>
    {
        // TODO: Move precedence relation validation to OperationalParameters to align with Tactical actor
        // Ensure work_order_activity start time is greater than previous activity finish time
        //
        for (
            window_index,
            ((_woa_0, operational_assignment_0), (_woa_1, operational_assignment_1)),
        ) in self
            .scheduled_work_order_activities
            .iter()
            .collect::<Vec<_>>()
            .windows(2)
            .map(|x| (&x[0], &x[1]))
            .enumerate()
        {
            if let ControlFlow::Break(_) =
                self.check_precedence_constraint(work_order_activity, window_index)
            {
                continue;
            }

            // TODO: Handle StartStart and Postpone activity relations
            // TODO: Validate activity_relation constraints during insertion
            let start_of_solution_window = operational_assignment_0.finish_time();

            let end_of_solution_window = operational_assignment_1.start_time();

            if start_of_solution_window
                < assignments
                    .first()
                    .expect("No Assignment in the OperationalSolution")
                    .start
                && assignments.last().unwrap().finish < end_of_solution_window
            {
                let operational_solution = OperationalAssignment::new(assignments);

                if !self.is_operational_solution_already_scheduled(work_order_activity) {
                    self.scheduled_work_order_activities.insert(
                        window_index + 1,
                        (work_order_activity, operational_solution),
                    );
                    let assignments = self
                        .scheduled_work_order_activities
                        .iter()
                        .flat_map(|(_, os)| &os.assignments)
                        .collect();

                    assert!(no_overlap_by_ref(assignments));
                }
                return None;
            }
        }

        Some(work_order_activity)
    }

    fn check_precedence_constraint(
        &self,
        work_order_activity: (WorkOrderNumber, u64),
        window_index: usize,
    ) -> ControlFlow<()>
    {
        let mut smallest: (usize, u64) = (usize::MIN, u64::MIN);
        let mut largest: (usize, u64) = (usize::MAX, u64::MAX);
        for (solution_index, work_order_activity_solution) in
            self.scheduled_work_order_activities.iter().enumerate()
        {
            if work_order_activity_solution.0.0 == work_order_activity.0 {
                if work_order_activity_solution.0.1 < work_order_activity.1
                    && smallest.1 < work_order_activity_solution.0.1
                {
                    smallest = (solution_index, work_order_activity_solution.0.1);
                } else if work_order_activity.1 < work_order_activity_solution.0.1
                    && work_order_activity_solution.0.1 < largest.1
                {
                    largest = (solution_index, work_order_activity_solution.0.1)
                }
            }
        }

        if window_index < smallest.0 || largest.0 <= window_index {
            return ControlFlow::Break(());
        }
        ControlFlow::Continue(())
    }

    pub fn containing_operational_solution(&self, time: DateTime<Utc>) -> ContainOrNextOrNone
    {
        let containing: Option<OperationalAssignment> = self
            .scheduled_work_order_activities
            .iter()
            .find(|operational_solution| operational_solution.1.contains(time))
            .map(|(_, os)| os)
            .cloned();

        match containing {
            Some(containing) => ContainOrNextOrNone::Contain(containing),
            None => {
                let next: Option<OperationalAssignment> = self
                    .scheduled_work_order_activities
                    .iter()
                    .map(|os| os.1.clone())
                    .find(|start| start.start_time() > time);

                match next {
                    Some(operational_solution) => ContainOrNextOrNone::Next(operational_solution),
                    None => ContainOrNextOrNone::None,
                }
            }
        }
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct OperationalAssignment
{
    /// Marginal fitness value, stored per WorkOrderActivity in the solution
    pub(crate) marginal_fitness: MarginalFitness,
    pub(crate) assignments: Vec<Assignment>,
}

impl OperationalAssignment
{
    pub fn new(assignments: Vec<Assignment>) -> Self
    {
        Self {
            assignments,
            marginal_fitness: MarginalFitness::default(),
        }
    }

    /// Return start time of the first assignment
    pub fn start_time(&self) -> DateTime<Utc>
    {
        self.assignments.first().unwrap().start
    }

    pub fn active_datetimes(&self) -> Vec<NaiveDate>
    {
        self.assignments
            .iter()
            .map(|e| e.start.date_naive())
            .collect()
    }

    pub fn finish_time(&self) -> DateTime<Utc>
    {
        self.assignments.last().unwrap().finish
    }

    pub fn contains(&self, time: DateTime<Utc>) -> bool
    {
        self.start_time() <= time && time < self.finish_time()
    }
}

// TODO: Move behavior to SharedSolutionTrait once code is stabilized
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Assignment
{
    pub operational_events: OperationalEvents,
    pub start: DateTime<Utc>,
    pub finish: DateTime<Utc>,
}

// NOTE: Ensure single source of truth for assignment time constraints
impl Assignment
{
    pub fn new(
        event_type: OperationalEvents,
        start: DateTime<Utc>,
        finish: DateTime<Utc>,
    ) -> Result<Self>
    {
        ensure!(
            event_type.time_delta() == finish - start,
            format!(
                "EventType: {:?}\nstart: {}\nfinish: {}",
                event_type, start, finish
            )
        );
        ensure!(
            start < finish,
            format!(
                "EventType: {:?}\nstart: {}\nfinish: {}",
                event_type, start, finish
            )
        );
        ensure!(
            event_type.start_time() == start.time(),
            format!(
                "EventType: {:?}\nstart: {}\nfinish: {}",
                event_type, start, finish
            )
        );
        ensure!(
            event_type.finish_time() == finish.time(),
            format!(
                "EventType: {:?}\nstart: {}\nfinish: {}",
                event_type, start, finish
            )
        );
        Ok(Self {
            operational_events: event_type,
            start,
            finish,
        })
    }

    pub fn make_unavailable_event(kind: Unavailability, availability: &Availability)
    -> Result<Self>
    {
        match kind {
            Unavailability::Beginning => {
                let event_start_time = availability
                    .start_datetime()
                    .clone()
                    .date_naive()
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_utc();
                let event_finish_time = availability.start_datetime();

                Assignment::new(
                    OperationalEvents::Unavailable(TimeInterval::from_date_times(
                        event_start_time,
                        event_finish_time,
                    )),
                    event_start_time,
                    event_finish_time,
                )
            }
            Unavailability::End => {
                let event_start_time = availability.finish_datetime();
                let event_finish_time = availability
                    .finish_datetime()
                    .clone()
                    .date_naive()
                    .and_hms_opt(23, 59, 59)
                    .unwrap()
                    .and_utc();

                Assignment::new(
                    OperationalEvents::Unavailable(TimeInterval::from_date_times(
                        event_start_time,
                        event_finish_time,
                    )),
                    event_start_time,
                    event_finish_time,
                )
            }
        }
    }
}

#[cfg(test)]
mod tests
{
    use std::ops::ControlFlow;

    use ordinator_orchestrator_actor_traits::marginal_fitness::MarginalFitness;
    use ordinator_scheduling_environment::work_order::WorkOrderNumber;

    use crate::algorithm::operational_solution::OperationalAssignment;
    use crate::algorithm::operational_solution::OperationalSolution;

    #[test]
    fn test_check_precedence_constraint()
    {
        let scheduled_work_order_activities = vec![
            (
                (WorkOrderNumber(2233990001), 10),
                OperationalAssignment {
                    marginal_fitness: MarginalFitness::None,
                    assignments: vec![],
                },
            ),
            (
                (WorkOrderNumber(2233990002), 20),
                OperationalAssignment {
                    marginal_fitness: MarginalFitness::None,
                    assignments: vec![],
                },
            ),
            (
                (WorkOrderNumber(2233990001), 30),
                OperationalAssignment {
                    marginal_fitness: MarginalFitness::None,
                    assignments: vec![],
                },
            ),
            (
                (WorkOrderNumber(2233990003), 40),
                OperationalAssignment {
                    marginal_fitness: MarginalFitness::None,
                    assignments: vec![],
                },
            ),
            (
                (WorkOrderNumber(2233990001), 50),
                OperationalAssignment {
                    marginal_fitness: MarginalFitness::None,
                    assignments: vec![],
                },
            ),
            (
                (WorkOrderNumber(2233990004), 60),
                OperationalAssignment {
                    marginal_fitness: MarginalFitness::None,
                    assignments: vec![],
                },
            ),
        ];
        let operational_solution = OperationalSolution {
            objective_value: super::OperationalObjectiveValue {
                hands_on_tool_time: 0,
                assess: 0,
                assign: 0,
                total_work_order_activities: 0,
            },
            scheduled_work_order_activities,
            non_productive: vec![],
        };

        let work_order_activity = (WorkOrderNumber(2233990001), 35);

        let window_indices = [0, 1, 2, 3, 4];
        let result = [
            ControlFlow::Break(()),
            ControlFlow::Break(()),
            ControlFlow::Continue(()),
            ControlFlow::Continue(()),
            ControlFlow::Break(()),
        ];

        for window_index in window_indices.iter().enumerate() {
            let control_flow = operational_solution
                .check_precedence_constraint(work_order_activity, *window_index.1);

            assert!(control_flow == result[window_index.0])
        }
    }
    #[test]
    fn test_check_precedence_constraint_1()
    {
        let scheduled_work_order_activities = vec![
            (
                (WorkOrderNumber(0), 0),
                OperationalAssignment {
                    marginal_fitness: MarginalFitness::None,
                    assignments: vec![],
                },
            ),
            (
                (WorkOrderNumber(0), 0),
                OperationalAssignment {
                    marginal_fitness: MarginalFitness::None,
                    assignments: vec![],
                },
            ),
        ];
        let operational_solution = OperationalSolution {
            objective_value: super::OperationalObjectiveValue {
                hands_on_tool_time: 0,
                assess: 0,
                assign: 0,
                total_work_order_activities: 0,
            },
            scheduled_work_order_activities,
            non_productive: vec![],
        };

        let work_order_activity = (WorkOrderNumber(2233990001), 35);

        let window_indices = [0];
        let result = [ControlFlow::Continue(())];

        for window_index in window_indices.iter().enumerate() {
            let control_flow = operational_solution
                .check_precedence_constraint(work_order_activity, *window_index.1);

            assert_eq!(control_flow, result[window_index.0])
        }
    }
}
