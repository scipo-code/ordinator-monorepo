pub mod delegate;
pub mod marginal_fitness;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::HashSet;
use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::MutexGuard;

use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use arc_swap::ArcSwap;
use bus::BusReader;
use chrono::NaiveDate;
use colored::Colorize;
use delegate::Delegate;
use flume::Receiver;
use flume::Sender;
use marginal_fitness::MarginalFitness;
use ordinator_configuration::SystemConfigurations;
use ordinator_scheduling_environment::SchedulingEnvironment;
use ordinator_scheduling_environment::time_environment::day::Day;
use ordinator_scheduling_environment::time_environment::period::Period;
use ordinator_scheduling_environment::work_order::WorkOrderActivity;
use ordinator_scheduling_environment::work_order::WorkOrderNumber;
use ordinator_scheduling_environment::work_order::operation::ActivityNumber;
use ordinator_scheduling_environment::work_order::operation::Work;
use ordinator_scheduling_environment::worker_environment::resources::ActorCompositeId;
use ordinator_scheduling_environment::worker_environment::resources::Skill;
use ordinator_scheduling_hypergraph::schedule_graph::SchedulingHypergraph;
use serde::Serialize;
use thiserror::Error;
use valuable::Valuable;

#[derive(Error, Debug)]
pub enum ActorError
{
    #[error("{info:#?}")]
    OptimizationError
    {
        info: ErrorInfo
    },
}

/// Trait for the clock that manages the whole system. This is a trait to
/// differentiate between the [`ProductionSystemClock`] and the
/// [`TestSystemClock`].
use std::fmt::Debug;
#[derive(Debug)]
pub struct ErrorInfo
{
    pub symptom: &'static str,
    pub hypothesis: &'static str,
    pub location: &'static std::panic::Location<'static>,
    pub action: &'static str,
    pub context: Box<dyn Debug + Send + Sync + 'static>,
}
pub struct Communication<RequestMessage, Res>
{
    sender_to_actor: Sender<RequestMessage>,
    pub receiver_from_actor: Receiver<Result<Res>>,
}

impl<RequestMessage, Res> Communication<RequestMessage, Res>
{
    pub fn new(sender: Sender<RequestMessage>, receiver: Receiver<Result<Res>>) -> Self
    {
        Self {
            sender_to_actor: sender,
            receiver_from_actor: receiver,
        }
    }

    pub fn from_agent(&self, message: RequestMessage) -> Result<()>
    {
        self.sender_to_actor.send(message).map_err(|e| anyhow!(e.to_string() )).context("The Actor has stopped running. If the reason for this is not obvious, it means that the error handling should be extended.")
    }

    pub fn from_actor(&self) -> Res
    {
        self.receiver_from_actor.recv().unwrap().unwrap()
    }
}

#[derive(PartialEq, Eq, Clone)]
pub struct SystemSolution<S, T, U, V>
where
    S: WeeklyInterface + Solution,
    T: ProjectInterface + Solution,
    U: DailyInterface + Solution,
    V: OperationalInterface + Solution,
{
    pub weekly: Option<SolutionState<S>>,
    pub project: Option<SolutionState<T>>,
    pub daily: Option<SolutionState<U>>,
    pub operational: HashMap<ActorCompositeId, SolutionState<V>>,
}

// ISSUE #000 [ ] - this whole [`SystemSolution`] should be reworked. It should
// be trait-based.
impl<S, T, U, V> std::fmt::Debug for SystemSolution<S, T, U, V>
where
    S: WeeklyInterface + Solution,
    T: ProjectInterface + Solution,
    U: DailyInterface + Solution,
    V: OperationalInterface + Solution,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        if f.alternate() {
            let number_of_operational_actors = self.operational.len().to_string();
            write!(
                f,
                "WeeklySolution:   {}\n\
                ProjectSolution:     {}\n\
                DailySolution:   {}\n\
                OperationalSolutions: {}",
                if self.weekly.is_some() {
                    "present".green()
                } else {
                    "absent".red()
                },
                if self.project.is_some() {
                    "present".green()
                } else {
                    "absent".red()
                },
                if self.daily.is_some() {
                    "present".green()
                } else {
                    "absent".red()
                },
                if !self.operational.is_empty() {
                    number_of_operational_actors.as_str().green()
                } else {
                    "absent".red()
                },
            )
        } else {
            panic!();
        }
    }
}

pub trait SystemSolutions: Clone + Sized
{
    type Weekly: WeeklyInterface;
    type Project: ProjectInterface;
    type Daily: DailyInterface;
    type Operational: OperationalInterface + Solution;

    fn new() -> Self;
    fn weekly(&self) -> Result<&Self::Weekly>;

    fn weekly_swap(&mut self, id: &ActorCompositeId, solution: SolutionState<Self::Weekly>)
    where
        Self::Weekly: Solution;
    fn project_actor_solution(&self) -> Result<&Self::Project>;

    fn project_swap(&mut self, id: &ActorCompositeId, solution: SolutionState<Self::Project>)
    where
        Self::Project: Solution;
    fn daily_actor_solutions(&self) -> Result<&Self::Daily>;

    fn daily_swap(&mut self, id: &ActorCompositeId, solution: SolutionState<Self::Daily>)
    where
        Self::Daily: Solution;
    fn operational_actor_solutions(&self, id: &ActorCompositeId) -> Result<&Self::Operational>;

    fn all_operational(&self) -> HashSet<ActorCompositeId>;

    fn operational_swap(
        &mut self,
        id: &ActorCompositeId,
        solution: SolutionState<Self::Operational>,
    ) where
        Self::Operational: Solution;
}

// TODO: Use trait composition instead of a single large trait.
#[allow(dead_code, unused_variables)]
impl<S, T, U, V> SystemSolutions for SystemSolution<S, T, U, V>
where
    S: WeeklyInterface + Solution,
    T: ProjectInterface + Solution,
    U: DailyInterface + Solution,
    V: OperationalInterface + Solution,
{
    type Daily = U;
    type Operational = V;
    type Project = T;
    type Weekly = S;

    fn new() -> Self
    {
        Self {
            weekly: None,
            project: None,
            daily: None,
            operational: HashMap::default(),
        }
    }

    fn weekly(&self) -> Result<&Self::Weekly>
    {
        Ok(&self
            .weekly
            .as_ref()
            .with_context(|| "WeeklyActor SystemSolution not found")?
            .inner)
    }

    fn project_actor_solution(&self) -> Result<&Self::Project>
    {
        Ok(&self
            .project
            .as_ref()
            .with_context(|| "ProjectActor SystemSolution not found")?
            .inner)
    }

    fn daily_actor_solutions(&self) -> Result<&Self::Daily>
    {
        Ok(&self
            .daily
            .as_ref()
            .with_context(|| "DailyActor SystemSolution not found")?
            .inner)
    }

    fn operational_actor_solutions(&self, id: &ActorCompositeId) -> Result<&Self::Operational>
    {
        Ok(&self
            .operational
            .get(id)
            .with_context(|| "OperationalActor SystemSolution not found")?
            .inner)
    }

    fn operational_swap(
        &mut self,
        id: &ActorCompositeId,
        solution: SolutionState<Self::Operational>,
    ) where
        Self::Operational: Solution,
    {
        self.operational.insert(id.clone(), solution);
    }

    fn weekly_swap(&mut self, id: &ActorCompositeId, solution: SolutionState<Self::Weekly>)
    where
        Self::Weekly: Solution,
    {
        self.weekly = Some(solution);
    }

    fn project_swap(&mut self, id: &ActorCompositeId, solution: SolutionState<Self::Project>)
    where
        Self::Project: Solution,
    {
        self.project = Some(solution);
    }

    fn daily_swap(&mut self, id: &ActorCompositeId, solution: SolutionState<Self::Daily>)
    where
        Self::Daily: Solution,
    {
        self.daily = Some(solution);
    }

    fn all_operational(&self) -> HashSet<ActorCompositeId>
    {
        self.operational.keys().cloned().collect()
    }
}

pub trait Parameters
where
    Self: Sized,
{
    type Key;

    /// Build parameters from a scheduling environment for a given actor
    fn from_scheduling_hypergraph(
        id: &ActorCompositeId,
        scheduling_environment: &MutexGuard<SchedulingHypergraph>,
    ) -> Result<Self>;

    /// Create and insert a new parameter. Note: this method can become
    /// extremely complex in practice
    fn create_and_insert_new_parameter(
        &mut self,
        key: Self::Key,
        scheduling_environment: MutexGuard<SchedulingEnvironment>,
    );

    // TODO: Add methods for updating configurations.
}

pub trait Solution: Sized + Debug
{
    type Objective: Debug + Valuable;
    type Parameters;

    /// Create a solution from parameters
    fn from_parameters(parameters: &Self::Parameters) -> Result<Self>;

    /// Update the solution's objective
    fn update_objective(&mut self, other_objective: Self::Objective);
}

pub type Stagnation = u64;
pub type Version = u64;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SolutionState<S: Solution>
{
    stagnation_iterations: Stagnation,
    version: Version,
    inner: S,
}

impl<S: Solution> Deref for SolutionState<S>
{
    type Target = S;

    fn deref(&self) -> &Self::Target
    {
        &self.inner
    }
}

impl<S: Solution> DerefMut for SolutionState<S>
{
    fn deref_mut(&mut self) -> &mut Self::Target
    {
        &mut self.inner
    }
}

/// Wrapper of [`Solution`] traits. Keeps track of the
/// number of iterations in the current [`Solution`]
/// with `stagnation_iterations` and keeps track of the
/// version of the best solution with `version`.
impl<S: Solution> SolutionState<S>
{
    pub fn new(solution: S) -> Self
    {
        SolutionState {
            stagnation_iterations: Stagnation::MIN,
            version: Version::MIN,
            inner: solution,
        }
    }

    pub fn revert_to_old_solution(&mut self, solution: S)
    {
        self.stagnation_iterations += 1;
        self.inner = solution;
    }

    pub fn update_objective(&mut self, objective: S::Objective)
    {
        self.stagnation_iterations = 0;
        self.version += 1;
        self.inner.update_objective(objective);
    }

    pub fn inner_mut(&mut self) -> &mut S
    {
        &mut self.inner
    }

    pub fn inner(&self) -> &S
    {
        &self.inner
    }

    pub fn stagnation_and_version(&self) -> (Stagnation, Version)
    {
        (self.stagnation_iterations, self.version)
    }
}

/// Handle incoming messages for an actor using a CQRS pattern
/// (reads via Orchestrator, commands via MessageHandler channel)
pub trait CommandHandler<Req, Res>
{
    fn handle_state_link(&mut self, state_link: StateLink) -> Result<Res>;

    fn handle_request_message(&mut self, request_message: Req) -> Result<Res>;
}

// TODO: Create a common solution interface for all levels (Weekly, Project,
// Daily, Operational)
pub trait WeeklyInterface
where
    Self: Clone + std::fmt::Debug + Eq + PartialEq,
{
    fn scheduled_task(
        &self,
        work_order_number: &WorkOrderNumber,
    ) -> Option<&WhereIsWorkOrder<Period>>;

    fn daily_tasks(&self, periods: &[Period]) -> HashMap<WorkOrderNumber, Period>;

    fn all_scheduled_tasks(&self) -> HashMap<WorkOrderNumber, Period>;
}

pub trait ProjectInterface
where
    Self: Clone + std::fmt::Debug + Eq + PartialEq,
{
    fn start_and_finish_dates(
        &self,
        work_order_activity: &WorkOrderActivity,
    ) -> Option<(&NaiveDate, &NaiveDate)>;

    fn project_period<'a>(
        &self,
        work_order_number: &WorkOrderNumber,
        periods: &'a [Period],
    ) -> Option<&'a Period>;

    fn all_scheduled_tasks(&self) -> HashMap<WorkOrderNumber, BTreeMap<ActivityNumber, Day>>;

    fn project_loadings(&self) -> BTreeMap<Skill, Vec<Work>>;
}

/// Indicates the scheduling level at which a work order is allocated.
/// Allows actors to implement custom logic based on work order location.
#[derive(PartialEq, Eq, Debug, Default, Clone, Serialize)]
pub enum WhereIsWorkOrder<T>
{
    Weekly(Period),
    Project(T),
    #[default]
    NotScheduled,
}
impl<T> WhereIsWorkOrder<T>
{
    pub fn is_project(&self) -> bool
    {
        matches!(self, WhereIsWorkOrder::Project(_))
    }

    pub fn not_scheduled(&self) -> bool
    {
        matches!(self, WhereIsWorkOrder::NotScheduled)
    }

    pub fn weekly_forced(&self) -> bool
    {
        matches!(self, WhereIsWorkOrder::Weekly(_))
    }

    pub fn is_weekly_or_project(&self) -> bool
    {
        self.is_project() || self.weekly_forced()
    }
}

pub trait SwapSolution<Ss>: Solution + Sized
where
    Ss: SystemSolutions,
{
    fn swap(id: &ActorCompositeId, solution: SolutionState<Self>, system_solution: &mut Ss);

    // fn perform_swap(id: &Id, solution: Self, system_solution:
    // Self::SystemSolution) {
    //     Self::swap(id, solution, system_solution);
    // }
}

pub trait DailyInterface
where
    Self: Clone + std::fmt::Debug + Eq + PartialEq,
{
    fn delegated_tasks(&self, operational_agent: &ActorCompositeId) -> HashSet<WorkOrderActivity>;

    /// Get delegates assigned to an operational agent
    fn delegates_for_agent(
        &self,
        operational_agent: &ActorCompositeId,
    ) -> HashMap<WorkOrderActivity, Delegate>;

    /// Count delegate types for an operational agent
    fn count_delegate_types(&self, operational_agent: &ActorCompositeId) -> (u64, u64, u64);
}

pub trait OperationalInterface
where
    Self: Clone + std::fmt::Debug + Eq + PartialEq,
{
    /// Get marginal fitness for a work order activity
    fn marginal_fitness_for_operational_actor<'a>(
        &'a self,
        work_order_activity: &WorkOrderActivity,
    ) -> Option<&'a MarginalFitness>;

    /// Get all scheduled activities for this operational actor
    fn scheduled_activities_for_operational_actor(&self) -> HashSet<WorkOrderActivity>;
}

/// The StateLink is a generic type that each type of Agent will implement.
/// The generics mean:
///     S: Weekly
///     T: Project
///     Su: Daily
///     O: Operational
/// This means that each Agent in the system will need to implement how to
/// understand messages from the other Agents in their own unique way.
/// This allows us to get custom implementations for each of the
/// Agent types creating a mesh of communication pathways that are still
/// statically typed.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum StateLink
{
    WorkOrders(Vec<WorkOrderNumber>),
    WorkerEnvironment,
    TimeEnvironment,
}

#[derive(Debug, Clone)]
pub enum ActorSpecific
{
    Weekly(Vec<WorkOrderNumber>),
}

pub trait ActorFactory<Ss>
where
    Ss: SystemSolutions + Sync + Send,
{
    type Communication;

    fn construct_actor(
        id: ActorCompositeId,
        scheduling_environment: Arc<Mutex<SchedulingHypergraph>>,
        system_solution_arc_swap: Arc<ArcSwap<Ss>>,
        system_configurations: Arc<ArcSwap<SystemConfigurations>>,
        stake_link_bus: BusReader<StateLink>,
        error_channel: Sender<anyhow::Error>,
    ) -> Result<Self::Communication>;
}

#[cfg(test)]
mod tests
{
    use crate::Solution;
    use crate::SolutionState;

    #[test]
    fn test_solution_state_increment()
    {
        #[derive(Debug)]
        struct TestSolution;

        impl Solution for TestSolution
        {
            type Objective = u64;
            type Parameters = ();

            fn from_parameters(_parameters: &Self::Parameters) -> anyhow::Result<Self>
            {
                todo!();
            }

            fn update_objective(&mut self, _other_objective: Self::Objective) {}
        }
        let mut solution = SolutionState::new(TestSolution);

        let objective = 9;

        solution.update_objective(objective);

        assert!(solution.version == 1);
        solution.update_objective(objective);

        assert!(solution.version == 2);

        assert!(solution.stagnation_iterations == 0);
        solution.revert_to_old_solution(TestSolution);
        assert!(solution.stagnation_iterations == 1);

        solution.revert_to_old_solution(TestSolution);
        assert!(solution.stagnation_iterations == 2);

        solution.revert_to_old_solution(TestSolution);
        assert!(solution.stagnation_iterations == 3);

        solution.revert_to_old_solution(TestSolution);
        assert!(solution.stagnation_iterations == 4);

        solution.revert_to_old_solution(TestSolution);
        assert!(solution.stagnation_iterations == 5);
        assert!(solution.version == 2);
        assert_ne!(solution.stagnation_iterations, 6);
    }
}
