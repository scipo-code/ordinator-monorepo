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
use ordinator_orchestrator_actor_traits::Parameters;
use ordinator_orchestrator_actor_traits::Solution;
use ordinator_orchestrator_actor_traits::StateLink;
use ordinator_orchestrator_actor_traits::SwapSolution;
use ordinator_orchestrator_actor_traits::SystemSolutions;
use ordinator_scheduling_environment::SchedulingEnvironment;
use ordinator_scheduling_environment::worker_environment::resources::ActorCompositeId;
use serde::Deserialize;
use serde::Serialize;
use tracing::Level;
use tracing::event;

use self::traits::ActorBasedLargeNeighborhoodSearch;

// I do not know if there is
// TODO [ ] FIX [ ]
// You should reuse the trait bounds on the Agent and the Algorithm.
//
pub struct Actor<ActorRequest, ActorResponse, Algorithm>
where
    // What should you do here with the
    // You should implement the MessageHandler for all of the
    // Actors this means that you need to create a blanket
    // implementation, and then the actors will have to supply
    // an implementation of the functions needed to actually
    // perform the required operations.
    // TODO [ ]
    // Look into whether it is possible for you to make a
    // blanket implementation that simply makes the
    // Actor implementations provide functions.
    Self: CommandHandler<ActorRequest, ActorResponse>,
    Algorithm: ActorBasedLargeNeighborhoodSearch + Debug,
    ActorResponse: Debug,
{
    pub actor_id: ActorCompositeId,
    pub scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
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

// TODO [ ]
// You should consider making a trait here for the agent. That is the best way
// of coding this. You are getting the hang of this and that is the most
// important thing here
impl<ActorRequest, ActorResponse, Algorithm> Actor<ActorRequest, ActorResponse, Algorithm>
where
    // This is
    // not possible.
    // It cannot
    // be implemented
    // like this.
    // That is
    // problem.
    // You
    // were confused
    // about this
    // before.
    // At least
    // now you
    // understand
    // the
    // implications of it.
    Self: CommandHandler<ActorRequest, ActorResponse>,
    Algorithm: ActorBasedLargeNeighborhoodSearch + Debug,
    ActorRequest: Send + Sync + 'static,
    ActorResponse: Send + Sync + 'static + Debug,
{
    // This method sends errors to the Orchestrator, which handles the errors
    // from there.
    //
    // One thing is for sure. Now is not the time to fix this.
    pub fn run(&mut self)
    {
        let mut schedule_iteration = ScheduleIteration::default();

        // I do not understand what I should be doing here? I think that the best
        // approach is to understand this as well as I can.

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

        // There is something fundamental that you are not getting here.
        loop {
            while let Ok(state_link) = self.state_link_receiver.try_recv() {
                match self.handle_state_link(state_link) {
                    // TODO [ ] 2025-07-15 This message could be used to communicate with
                    // the Orchestrator again.
                    Ok(_e) => {
                        event!(target: "business_event", Level::INFO, "{}", format!("Actor {} handled a state_link_message\nActorResponse {_e:?}",self.actor_id));
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

            let sleep_duration = match self
                .configurations
                .load()
                .throttling
                .get_throttling(&self.actor_id.0)
            {
                Ok(throttling) => throttling,
                Err(err) => {
                    self.error_channel
                        .send(err)
                        .expect("If error channel is down, everything is down");
                    9999
                }
            };

            std::thread::sleep(std::time::Duration::from_millis(sleep_duration));
            if let Err(actor_error) = self
                .algorithm
                // Ahh the issue is that you cannot put this kind of thing in here. The issue comes
                // from the fact that the. The Actor needs to run this.
                // Should the Option be removed? Yes
                .run_lns_iteration()
                .with_context(|| {
                    format!(
                        "{schedule_iteration:#?}\nActor: {}\nLocation: {}",
                        self.actor_id,
                        Location::caller(),
                    )
                })
            {
                self.error_channel
                    .send(actor_error)
                    .expect("If this happens no amount of error handling will save the program")
            }

            schedule_iteration.increment();
        }
    }

    // I believe that many of these fields can be set by themselves.
    //
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

// Is what you are getting from this worth it? I do not really
// think so. You will have to make a new function in the
// other
pub struct ActorBuilder<ActorRequest, ActorResponse, Algorithm>
where
    Algorithm: ActorBasedLargeNeighborhoodSearch,
    ActorRequest: Send + Sync + 'static,
    ActorResponse: Send + Sync + 'static,
{
    agent_id: Option<ActorCompositeId>,
    scheduling_environment: Option<Arc<Mutex<SchedulingEnvironment>>>,
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
        scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
    ) -> Self
    {
        self.scheduling_environment = Some(scheduling_environment);
        self
    }

    // QUESTION [ ]
    // Do you actually want the `From` trait bound here?
    //
    // What are the alternative options here? I think that the best
    // thing to do
    // Algorithmh call `builder` itself. You should not have to do much.
    pub fn algorithm<F, S, P, I, Ss>(mut self, configure: F) -> Result<Self>
    where
        SpecificAlgorithm: From<algorithm::Algorithm<S, P, I, Ss>>,
        // I do not think that this should be implemented on the
        // You are over engineering it here but I do not see what
        // other options that we have for making this a success.
        S: Solution<Parameters = P> + Debug + Clone + SwapSolution<Ss>,
        Ss: SystemSolutions,
        P: Parameters,
        I: Default,
        F: FnOnce(AlgorithmBuilder<S, P, I, Ss>) -> Result<AlgorithmBuilder<S, P, I, Ss>>,
    {
        let algorithm_builder = algorithm::Algorithm::builder();

        let algorithm_builder = configure(algorithm_builder)?;

        self.algorithm = Some(SpecificAlgorithm::from(algorithm_builder.build()?));

        Ok(self)
    }

    // What is the error here? I think that it has to do with the
    // bounded channel.
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

/// This type is the primary message type that all agents should receive.
/// All agents should have the `StateLink` and each agent then have its own
/// ActorRequest which is specifically created for each agent.
// THIS should most likely be removed or refactored.
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

// You need type safety here I do not see another way around it
//
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
