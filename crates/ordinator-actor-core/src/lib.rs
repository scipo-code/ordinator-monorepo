pub mod algorithm;
pub mod traits;

use std::fmt::Debug;
use std::fmt::{self};
use std::panic::Location;
use std::sync::Arc;
use std::sync::Mutex;

use algorithm::AlgorithmBuilder;
use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use arc_swap::ArcSwap;
use bus::BusReader;
use colored::Colorize;
use flume::Receiver;
use flume::Sender;
use ordinator_configuration::SystemConfigurations;
use ordinator_orchestrator_actor_traits::CommandHandler;
use ordinator_orchestrator_actor_traits::Communication;
use ordinator_orchestrator_actor_traits::Options;
use ordinator_orchestrator_actor_traits::Parameters;
use ordinator_orchestrator_actor_traits::Solution;
use ordinator_orchestrator_actor_traits::StateLink;
use ordinator_orchestrator_actor_traits::SwapSolution;
use ordinator_orchestrator_actor_traits::SystemSolutions;
use ordinator_scheduling_environment::SchedulingEnvironment;
use ordinator_scheduling_environment::worker_environment::resources::ActorCompositeId;
use ordinator_scheduling_hypergraph::schedule_graph::SchedulingHypergraph;
use serde::Deserialize;
use serde::Serialize;
use tracing::Level;
use tracing::event;
use tracing::info;

use self::traits::ActorBasedLargeNeighborhoodSearch;

// TODO: Reuse trait bounds on the Agent and the Algorithm.
pub struct Actor<ActorRequest, ActorResponse, Algorithm>
where
    // TODO: Consider implementing a blanket MessageHandler
    // trait that Actor implementations provide through a
    // custom interface.
    Self: CommandHandler<ActorRequest, ActorResponse>,
    Algorithm: ActorBasedLargeNeighborhoodSearch + Debug,
    ActorResponse: Debug,
{
    pub actor_id: ActorCompositeId,
    // Should this be in here. No! The Actors and algorithms can have no understanding of the
    // `SchedulingEnvironment` You have to build the SchedulingHypergraph from the
    // SchedulingEnvironment... This will lead to another problem, what if the change in the
    // Hypergraph requires the ERP system to change?
    //
    // I think that the SchedulingHypergraph should contain all constrints and then the
    // system should tell the repositories in the SchedulingEnvironment how they should change.
    pub scheduling_environment: Arc<Mutex<SchedulingHypergraph>>,
    pub algorithm: Algorithm,
    // TODO [ ] 2025-07-14 These senders and receivers are relevant for the `Algorithm`
    // and not shared with the `SchedulingEnvironment` changes and the `StateLink` and
    // should therefore be changed.
    pub receiver_from_orchestrator: Receiver<ActorRequest>,
    pub sender_to_orchestrator: Sender<Result<ActorResponse>>,
    pub state_link_receiver: BusReader<StateLink>,
    pub configurations: Arc<ArcSwap<SystemConfigurations>>,
    pub error_channel: Sender<anyhow::Error>,
}

// TODO: Consider implementing a trait for the agent as the primary interface.
impl<ActorRequest, ActorResponse, Algorithm> Actor<ActorRequest, ActorResponse, Algorithm>
where
    Self: CommandHandler<ActorRequest, ActorResponse>,
    Algorithm: ActorBasedLargeNeighborhoodSearch + Debug,
    ActorRequest: Send + Sync + 'static,
    ActorResponse: Send + Sync + 'static + Debug,
{
    // Run the actor's main event loop, handling algorithm scheduling and
    // orchestrator communication.
    pub fn run(&mut self)
    {
        info!(target: "developer", "CHECK THAT EVERY ALGORITHM IS HERE");
        let mut schedule_iteration = ScheduleIteration::default();

        if let Err(actor_error) = self.algorithm.schedule().with_context(|| {
            format!(
                "{schedule_iteration:#?}\n\
                Actor    : {:#?}\n\
                Algorithm: {:#?}\n\
                Location : {}",
                self.actor_id,
                self.algorithm,
                Location::caller(),
            )
        }) {
            self.error_channel
                .send(anyhow!(actor_error))
                .expect("If this happens no amount of error handling will save the program")
        }

        schedule_iteration.increment();

        loop {
            while let Ok(state_link) = self.state_link_receiver.try_recv() {
                match self.handle_state_link(state_link) {
                    // TODO [ ] 2025-07-15 This message could be used to communicate with
                    // the Orchestrator again.
                    Ok(_e) => {
                        event!(target: "business_event", Level::INFO, "{:?}", format!("Actor {} handled a state_link_message\nActorResponse {_e:?}",self.actor_id));
                    }
                    Err(e) => self.error_channel.send(e).expect(
                        "If this happens no amount of error handling will save the program",
                    ),
                }
            }

            while let Ok(message) = self.receiver_from_orchestrator.try_recv() {
                match self.handle_request_message(message) {
                    Ok(_) => (),
                    Err(e) => self.error_channel.send(e).expect(
                        "If this happens no amount of error handling will save the program",
                    ),
                }
            }

            let throttling = &self.configurations.load().throttling;

            let sleep_duration = self.algorithm.throttling(throttling);

            std::thread::sleep(std::time::Duration::from_millis(sleep_duration));
            if let Err(actor_error) = self.algorithm.run_lns_iteration().with_context(|| {
                format!(
                    "{schedule_iteration:#?}\n\
                    Actor    : {:#?}\n\
                    Algorithm: {:#?}\n\
                    Location : {}",
                    self.actor_id,
                    self.algorithm,
                    Location::caller(),
                )
            }) {
                self.error_channel
                    .send(anyhow!(actor_error))
                    .expect("If this happens no amount of error handling will save the program")
            }

            schedule_iteration.increment();
        }
    }

    /// Construct and spawn an actor with the given options, returning the
    /// communication channel to the orchestrator.
    pub fn construct<S, P, I, O, System>(
        id: ActorCompositeId,
        scheduling_hypergraph: Arc<Mutex<SchedulingHypergraph>>,
        system_solutions: Arc<ArcSwap<System>>,
        system_configurations: Arc<ArcSwap<SystemConfigurations>>,
        state_link_bus: BusReader<StateLink>,
        error_channel: Sender<anyhow::Error>,
        options: O,
    ) -> Result<Communication<ActorRequest, ActorResponse>>
    where
        Algorithm: From<algorithm::Algorithm<S, P, I, O, System>> + Send + 'static,
        S: Solution<Parameters = P> + Debug + Clone + SwapSolution<System>,
        System: SystemSolutions,
        P: Parameters<Options = O> + Debug,
        O: Options,
        I: Default,
    {
        Self::builder()
            .agent_id(id.clone())
            .scheduling_environment(Arc::clone(&scheduling_hypergraph))
            .algorithm(|ab| {
                ab.id(id)
                    .parameters_and_solution(&scheduling_hypergraph.lock().unwrap(), options)?
                    .system_solution_arc_swap(system_solutions)
            })?
            .communication(error_channel, state_link_bus)
            .configurations(system_configurations)
            .build()
    }

    // Create a new ActorBuilder for constructing Actor instances.
    pub fn builder() -> ActorBuilder<ActorRequest, ActorResponse, Algorithm>
    {
        ActorBuilder {
            agent_id: None,
            scheduling_environment: None,
            algorithm: None,
            receiver_from_orchestrator: None,
            sender_to_orchestrator: None,
            state_link_bus: None,
            configurations: None,
            communication_for_orchestrator: None,
            error_channel: None,
        }
    }
}

pub struct ActorBuilder<ActorRequest, ActorResponse, Algorithm>
where
    Algorithm: ActorBasedLargeNeighborhoodSearch,
    ActorRequest: Send + Sync + 'static,
    ActorResponse: Send + Sync + 'static,
{
    agent_id: Option<ActorCompositeId>,
    scheduling_environment: Option<Arc<Mutex<SchedulingHypergraph>>>,
    algorithm: Option<Algorithm>,
    receiver_from_orchestrator: Option<Receiver<ActorRequest>>,
    sender_to_orchestrator: Option<Sender<Result<ActorResponse>>>,
    state_link_bus: Option<BusReader<StateLink>>,
    configurations: Option<Arc<ArcSwap<SystemConfigurations>>>,
    //
    communication_for_orchestrator: Option<Communication<ActorRequest, ActorResponse>>,
    error_channel: Option<Sender<anyhow::Error>>,
}

impl<ActorRequest, ActorResponse, SpecificAlgorithm>
    ActorBuilder<ActorRequest, ActorResponse, SpecificAlgorithm>
where
    Actor<ActorRequest, ActorResponse, SpecificAlgorithm>:
        CommandHandler<ActorRequest, ActorResponse>,
    SpecificAlgorithm: ActorBasedLargeNeighborhoodSearch + Send + 'static + Debug,
    ActorRequest: Send + Sync + 'static,
    ActorResponse: Send + Sync + 'static + Debug,
{
    pub fn build(self) -> Result<Communication<ActorRequest, ActorResponse>>
    {
        let mut agent = Actor {
            actor_id: self.agent_id.unwrap(),
            scheduling_environment: self.scheduling_environment.unwrap(),
            algorithm: self.algorithm.unwrap(),
            receiver_from_orchestrator: self.receiver_from_orchestrator.unwrap(),
            sender_to_orchestrator: self.sender_to_orchestrator.unwrap(),
            state_link_receiver: self.state_link_bus.unwrap(),
            configurations: self.configurations.unwrap(),
            error_channel: self.error_channel.unwrap(),
        };

        let thread_name = agent.actor_id.0.to_string();

        std::thread::Builder::new()
            .name(thread_name)
            .spawn(move || agent.run())?;

        Ok(self.communication_for_orchestrator.unwrap())
    }

    pub fn agent_id(mut self, agent_id: ActorCompositeId) -> Self
    {
        self.agent_id = Some(agent_id);
        self
    }

    pub fn scheduling_environment(
        mut self,
        scheduling_environment: Arc<Mutex<SchedulingHypergraph>>,
    ) -> Self
    {
        self.scheduling_environment = Some(scheduling_environment);
        self
    }

    // Configure the algorithm for this actor.
    pub fn algorithm<F, S, P, I, O, Ss>(mut self, configure: F) -> Result<Self>
    where
        SpecificAlgorithm: From<algorithm::Algorithm<S, P, I, O, Ss>>,
        S: Solution<Parameters = P> + Debug + Clone + SwapSolution<Ss>,
        Ss: SystemSolutions,
        P: Parameters<Options = O> + Debug,
        O: Options,
        I: Default,
        F: FnOnce(AlgorithmBuilder<S, P, I, O, Ss>) -> Result<AlgorithmBuilder<S, P, I, O, Ss>>,
    {
        let algorithm_builder = algorithm::Algorithm::builder();

        let algorithm_builder = configure(algorithm_builder)?;

        self.algorithm = Some(SpecificAlgorithm::from(algorithm_builder.build()?));

        Ok(self)
    }

    // Set up communication channels with the orchestrator and state link bus.
    pub fn communication(
        mut self,
        error_channel: Sender<anyhow::Error>,
        bus_reader: BusReader<StateLink>,
    ) -> Self
    {
        let (sender_to_actor, receiver_from_orchestrator): (
            flume::Sender<ActorRequest>,
            flume::Receiver<ActorRequest>,
        ) = flume::unbounded();

        let (sender_to_orchestrator, receiver_from_actor): (
            flume::Sender<Result<ActorResponse>>,
            flume::Receiver<Result<ActorResponse>>,
        ) = flume::unbounded();

        self.communication_for_orchestrator =
            Some(Communication::new(sender_to_actor, receiver_from_actor));

        self.receiver_from_orchestrator = Some(receiver_from_orchestrator);
        self.sender_to_orchestrator = Some(sender_to_orchestrator);
        self.error_channel = Some(error_channel);
        self.state_link_bus = Some(bus_reader);
        self
    }

    pub fn receiver_from_orchestrator(
        mut self,
        receiver_from_orchestrator: Receiver<ActorRequest>,
    ) -> Self
    {
        self.receiver_from_orchestrator = Some(receiver_from_orchestrator);
        self
    }

    pub fn sender_to_orchestrator(
        mut self,
        sender_to_orchestrator: Sender<Result<ActorResponse>>,
    ) -> Self
    {
        self.sender_to_orchestrator = Some(sender_to_orchestrator);
        self
    }

    pub fn configurations(mut self, configurations: Arc<ArcSwap<SystemConfigurations>>) -> Self
    {
        self.configurations = Some(configurations);
        self
    }
}

#[derive(Default)]
pub struct ScheduleIteration
{
    loop_iteration: u64,
}

impl ScheduleIteration
{
    pub fn increment(&mut self)
    {
        self.loop_iteration += 1;
    }
}

impl fmt::Debug for ScheduleIteration
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        if f.alternate() {
            let string = format!(
                "{}: {}",
                std::any::type_name::<ScheduleIteration>()
                    .split("::")
                    .last()
                    .unwrap(),
                self.loop_iteration
            )
            .bright_magenta();

            write!(f, "{string}")
        } else {
            f.debug_struct("ScheduleIteration")
                .field("loop_iteration", &self.loop_iteration)
                .finish()
        }
    }
}

/// Represents the feasibility state of an algorithm's solution.
///
/// This is the primary message type for communicating algorithm state to
/// agents.
#[derive(Debug, Serialize)]
pub enum AlgorithmState<T>
{
    Feasible,
    Infeasible(T),
}

impl<T> AlgorithmState<T>
{
    pub fn infeasible_cases_mut(&mut self) -> Option<&mut T>
    {
        match self {
            AlgorithmState::Feasible => None,
            AlgorithmState::Infeasible(infeasible_cases) => Some(infeasible_cases),
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum ConstraintState<Reason>
{
    Feasible,
    Infeasible(Reason),
    Undetermined,
}

impl<Reason> fmt::Display for ConstraintState<Reason>
where
    Reason: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self {
            ConstraintState::Feasible => write!(f, "FEASIBLE"),
            ConstraintState::Infeasible(reason) => write!(f, "{reason}"),
            ConstraintState::Undetermined => write!(f, "Constraint is not determined yet"),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum RequestMessage<S, Sc, R, T, C>
{
    Status(S),
    Scheduling(Sc),
    Resource(R),
    Time(T),
    SchedulingEnvironment(C),
    Update,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum ResponseMessage<S, Sc, R, T, C>
{
    Status(S),
    Scheduling(Sc),
    Resource(R),
    Time(T),
    SchedulingEnvironment(C),
    Update,
    Succes,
}
