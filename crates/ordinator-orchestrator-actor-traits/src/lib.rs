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
use ordinator_scheduling_environment::worker_environment::resources::Resources;
use serde::Serialize;
use thiserror::Error;

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

// StateLink is not a request. It is something different
// Ahh this is good every Request message from each of the actors
// should implement a `RequestMessage`. It is a little weird to
// reuse the `Req` like this. You need to remember this to see
// what you will learn from it.
//
// You are misunderstanding this because you are not using the
// generics in the correct way. There is something to learn here.
impl<RequestMessage, Res> Communication<RequestMessage, Res>
{
    pub fn new(sender: Sender<RequestMessage>, receiver: Receiver<Result<Res>>) -> Self
    {
        Self {
            sender_to_actor: sender,
            receiver_from_actor: receiver,
        }
    }

    // This is being wrapped twice. I think that the best approach is to
    // make the system function with.
    pub fn from_agent(&self, message: RequestMessage) -> Result<()>
    {
        // What is it that you need to do here? You should
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
    S: StrategicInterface + Solution,
    T: TacticalInterface + Solution,
    U: SupervisorInterface + Solution,
    V: OperationalInterface + Solution,
{
    pub strategic: Option<SolutionState<S>>,
    pub tactical: Option<SolutionState<T>>,
    pub supervisor: Option<SolutionState<U>>,
    pub operational: HashMap<ActorCompositeId, SolutionState<V>>,
}

// ISSUE #000 [ ] - this whole [`SystemSolution`] should be reworked. It should
// be trait-based.
impl<S, T, U, V> std::fmt::Debug for SystemSolution<S, T, U, V>
where
    S: StrategicInterface + Solution,
    T: TacticalInterface + Solution,
    U: SupervisorInterface + Solution,
    V: OperationalInterface + Solution,
{
    // What you are doing here is so important! That you decided to make this is
    // one of the best idea that you have had in a long time!
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        if f.alternate() {
            let number_of_operational_actors = self.operational.len().to_string();
            write!(
                f,
                "StrategicSolution:   {}\n\
                TacticalSolution:     {}\n\
                SupervisorSolution:   {}\n\
                OperationalSolutions: {}",
                if self.strategic.is_some() {
                    "present".green()
                } else {
                    "absent".red()
                },
                if self.tactical.is_some() {
                    "present".green()
                } else {
                    "absent".red()
                },
                if self.supervisor.is_some() {
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

// This is made completely wrong. I am not sure what the
// best approach of solving it will be.
pub trait SystemSolutions: Clone + Sized
{
    type Strategic: StrategicInterface;
    type Tactical: TacticalInterface;
    type Supervisor: SupervisorInterface;
    type Operational: OperationalInterface + Solution;

    fn new() -> Self;
    fn strategic(&self) -> Result<&Self::Strategic>;

    fn strategic_swap(&mut self, id: &ActorCompositeId, solution: SolutionState<Self::Strategic>)
    where
        Self::Strategic: Solution;
    fn tactical_actor_solution(&self) -> Result<&Self::Tactical>;

    fn tactical_swap(&mut self, id: &ActorCompositeId, solution: SolutionState<Self::Tactical>)
    where
        Self::Tactical: Solution;
    fn supervisor_actor_solutions(&self) -> Result<&Self::Supervisor>;

    fn supervisor_swap(&mut self, id: &ActorCompositeId, solution: SolutionState<Self::Supervisor>)
    where
        Self::Supervisor: Solution;
    fn operational_actor_solutions(&self, id: &ActorCompositeId) -> Result<&Self::Operational>;

    fn all_operational(&self) -> HashSet<ActorCompositeId>;
    // If you make all Id's internal you could simply work on those?
    fn operational_swap(&mut self, id: &ActorCompositeId, solution: SolutionState<Self::Operational>)
    where
        Self::Operational: Solution;
}

// ISSUE #000 [ ] - use trait composition instead of a single large trait.
#[allow(dead_code, unused_variables)]
impl<S, T, U, V> SystemSolutions for SystemSolution<S, T, U, V>
where
    S: StrategicInterface + Solution,
    T: TacticalInterface + Solution,
    U: SupervisorInterface + Solution,
    V: OperationalInterface + Solution,
{
    type Operational = V;
    type Strategic = S;
    type Supervisor = U;
    type Tactical = T;

    fn new() -> Self
    {
        Self {
            strategic: None,
            tactical: None,
            supervisor: None,
            operational: HashMap::default(),
        }
    }

    fn strategic(&self) -> Result<&Self::Strategic>
    {
        Ok(&self
            .strategic
            .as_ref()
            .with_context(|| "StrategicActor SystemSolution not found")?
            .inner)
    }

    fn tactical_actor_solution(&self) -> Result<&Self::Tactical>
    {
        Ok(&self
            .tactical
            .as_ref()
            .with_context(|| "TacticalActor SystemSolution not found")?
            .inner)
    }

    fn supervisor_actor_solutions(&self) -> Result<&Self::Supervisor>
    {
        Ok(&self
            .supervisor
            .as_ref()
            .with_context(|| "SupervisorActor SystemSolution not found")?
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

    // Can you even do this? Is this allowed? I do not t
    fn operational_swap(&mut self, id: &ActorCompositeId, solution: SolutionState<Self::Operational>)
    where
        Self::Operational: Solution,
    {
        self.operational.insert(id.clone(), solution);
    }

    fn strategic_swap(&mut self, id: &ActorCompositeId, solution: SolutionState<Self::Strategic>)
    where
        Self::Strategic: Solution,
    {
        self.strategic = Some(solution);
    }

    fn tactical_swap(&mut self, id: &ActorCompositeId, solution: SolutionState<Self::Tactical>)
    where
        Self::Tactical: Solution,
    {
        self.tactical = Some(solution);
    }

    fn supervisor_swap(&mut self, id: &ActorCompositeId, solution: SolutionState<Self::Supervisor>)
    where
        Self::Supervisor: Solution,
    {
        self.supervisor = Some(solution);
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

    /// Who should build the parameters. That is the key question here.
    /// Do you want to mutate it?
    ///
    /// I really do not like this trait declaration. Something has to change?
    fn from_source(
        id: &ActorCompositeId,
        scheduling_environment: &MutexGuard<SchedulingEnvironment>,
    ) -> Result<Self>;

    /// WARNING
    /// This method can become extremely complex in a practical setting.
    /// You should do.
    fn create_and_insert_new_parameter(
        &mut self,
        key: Self::Key,
        scheduling_environment: MutexGuard<SchedulingEnvironment>,
    );

    // TODO [ ]
    // Add methods for updating configurations.
}

// There is something that I do not like about having `new` here
// I think that the best option is to make the system work with the
// `from` trait. Meaning that we should focus on making the system
// work with the
// Should this function have an option or not? Yes it should.
pub trait Solution: Sized + Debug
{
    type Objective: Debug;
    type Parameters;

    // The weightings are found inside of the
    // `Solution`
    // QUESTION
    // Is this a good idea to create the Solution? I actually believe that it
    // is!
    // Should you have the options here? I think that you should derive the...
    //
    // The solution should only contain the things that actually change.
    fn from_parameters(parameters: &Self::Parameters) -> Result<Self>;

    fn update_objective(&mut self, other_objective: Self::Objective);
}

pub type Iterations = u64;
pub type Version = u64;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SolutionState<S: Solution>
{
    stagnation_iterations: Iterations,
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
            stagnation_iterations: Iterations::MIN,
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
}

// NOTE [ ]
// You are actually turning this `MessageHandler` into a CQRS pattern. All reads
// will be done through the `Orchestrator` and only commands will be send
// through the `MassageHandler` channel.
/// This trait should be implemented by every Actor so that it will be able to
/// receive messages from the user and the [`Orchestrator`].  
///
/// Funny
/// TODO [ ] You need to experience so much pain for this to work correctly
pub trait CommandHandler<Req, Res>
{
    fn handle_state_link(&mut self, state_link: StateLink) -> Result<Res>;

    fn handle_request_message(&mut self, request_message: Req) -> Result<Res>;
}

// There should only be a single interface here there should be a
// a set of standard operations that every solution should inplement
// this is to make sure that you do not make stray impl blocks and
// TODO [ ]
// Make a solution interface that is common
pub trait StrategicInterface
where
    Self: Clone + std::fmt::Debug + Eq + PartialEq,
{
    fn scheduled_task(
        &self,
        work_order_number: &WorkOrderNumber,
    ) -> Option<&WhereIsWorkOrder<Period>>;

    fn supervisor_tasks(&self, periods: &[Period]) -> HashMap<WorkOrderNumber, Period>;

    fn all_scheduled_tasks(&self) -> HashMap<WorkOrderNumber, Period>;
}

pub trait TacticalInterface
where
    Self: Clone + std::fmt::Debug + Eq + PartialEq,
{
    fn start_and_finish_dates(
        &self,
        work_order_activity: &WorkOrderActivity,
    ) -> Option<(&NaiveDate, &NaiveDate)>;

    fn tactical_period<'a>(
        &self,
        work_order_number: &WorkOrderNumber,
        periods: &'a [Period],
    ) -> Option<&'a Period>;

    fn all_scheduled_tasks(&self) -> HashMap<WorkOrderNumber, BTreeMap<ActivityNumber, Day>>;

    fn tactical_loadings(&self) -> BTreeMap<Resources, Vec<Work>>;
}

// This is a core type that each `Actor` should implement, I think
// that it should be part of a trait but which is a little difficult
// to tell.
// QUESTION
// What is this type set in the world to do?
// The goal of it is to make sure that the `Actor` can make
// custom logic internally depending on where they know the
// work order to be located. This is crucial to respect
// business logic.
//
// ESSAY: 2025-07-21
// It is crucial that these types are correctly aligned here. I think
// that the best idea is to.. You should not use any resource if it
// is not in the StrategicActor.
#[derive(PartialEq, Eq, Debug, Default, Clone, Serialize)]
pub enum WhereIsWorkOrder<T>
{
    Strategic(Period),
    Tactical(T),
    #[default]
    NotScheduled,
}
impl<T> WhereIsWorkOrder<T>
{
    pub fn is_tactical(&self) -> bool
    {
        matches!(self, WhereIsWorkOrder::Tactical(_))
    }

    pub fn not_scheduled(&self) -> bool
    {
        matches!(self, WhereIsWorkOrder::NotScheduled)
    }

    pub fn strategic_forced(&self) -> bool
    {
        matches!(self, WhereIsWorkOrder::Strategic(_))
    }

    pub fn is_strategic_or_tactical(&self) -> bool
    {
        self.is_tactical() || self.strategic_forced()
    }
}

// NOTE
// One thing is for sure here. It does not make sense to
// have `Ss` and then only have SystemSolution in here.
// You have a dilemma here then. Either you make the
// trait Ss or you remove Ss from everywhere in the
// code. I can feel that is the right question to be
// asking.
//
// I will keep the [`Ss`]
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

pub trait SupervisorInterface
where
    Self: Clone + std::fmt::Debug + Eq + PartialEq,
{
    fn delegated_tasks(&self, operational_agent: &ActorCompositeId) -> HashSet<WorkOrderActivity>;
    // Where should the `Delegate` be located?
    // I believe that the best place is in the `actor-core` the issue is that you
    // wanted to use these interface for the `orchestrator` as well and that is not
    // what you actually want. You need the orchestrator to have the delegate as
    // well in general you would like to have the `Solutions` able to be
    // exported directly from the `orchestrator`.
    // QUESTION
    // TODO [ ]
    // Should you simply move the module into this crate? Yes I think that is a good
    // idea.
    // What should the `delegates` here be called? Remember that you can use
    // associated types to fix this in the correct way. The best approach would
    // QUESTION
    // What exactly is the function doing? It is for a specific actor finding every
    // work order activity that is relevant for this actor. The function is
    // basically returning a `SortedSolution` for the `SupervisorSolution`.
    //
    // I think...
    // The best approach here is to make something that we make a trait for each and
    // then afterwards you look at all four of these traits and then make a
    // common! Bullseye! This is the approach abstract should always be created
    // with evidence not blind faith.
    fn delegates_for_agent(&self, operational_agent: &ActorCompositeId) -> HashMap<WorkOrderActivity, Delegate>;
    fn count_delegate_types(&self, operational_agent: &ActorCompositeId) -> (u64, u64, u64);
}
// The `solution` should be updated on the `SharedSolution` not the
// individual solution. These interfaces are implemented on the
// individual solution.
pub trait OperationalInterface
where
    Self: Clone + std::fmt::Debug + Eq + PartialEq,
{
    // This function is completely on the wrong level of abstraction, this
    // feeling is what should guide you towards correct behavior.
    // This interface should be implemented by the `operational` actor.
    // And this means that the most important thing is the that the
    // method cannot see the solution from the `supervisor` are you missing
    // something here?
    //
    // This should not be a `Vec` correct? A `WorkOrderActivity` is unique to
    // this actor? Yes
    fn marginal_fitness_for_operational_actor<'a>(
        &'a self,
        work_order_activity: &WorkOrderActivity,
    ) -> Option<&'a MarginalFitness>;

    fn scheduled_activities_for_operational_actor(&self) -> HashSet<WorkOrderActivity>;
}

/// The StateLink is a generic type that each type of Agent will implement.
/// The generics mean:
///     S: Strategic
///     T: Tactical
///     Su: Supervisor
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

// You can now remove it. The actors should never communicate state like
// this! This is a bad way of... Maybe not... Knowing the `Actor` that
// caused this message could be valuable... But I do not think so.
//
// This message was made when the idea was that the `Actor`s themselves
// should create this. You are `Simplifying`!
#[derive(Debug, Clone)]
pub enum ActorSpecific
{
    Strategic(Vec<WorkOrderNumber>),
}

pub trait ActorFactory<Ss>
where
    Ss: SystemSolutions + Sync + Send,
{
    type Communication;

    fn construct_actor(
        id: ActorCompositeId,
        scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
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
                todo!()
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
