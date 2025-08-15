mod actor_factory;
pub mod actor_registry;
pub mod database;
pub mod logging;
pub(crate) mod system_solution_tester;

use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::Arc;

use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use anyhow::bail;
use arc_swap::ArcSwap;
use chrono::DateTime;
use chrono::Utc;
use flume::Receiver;
use flume::Sender;
use ordinator_configuration::SystemConfigurations;
use ordinator_contracts::orchestrator::OrchestratorResponse;
use ordinator_operational_actor::OperationalApi;
use ordinator_operational_actor::algorithm::operational_solution::OperationalSolution;
pub use ordinator_operational_actor::messages::OperationalRequestMessage;
pub use ordinator_operational_actor::messages::OperationalResponseMessage;
pub use ordinator_operational_actor::messages::requests::OperationalStatusRequest;
use ordinator_orchestrator_actor_traits::ActorFactory;
use ordinator_orchestrator_actor_traits::Communication;
pub use ordinator_orchestrator_actor_traits::StateLink;
pub use ordinator_orchestrator_actor_traits::StrategicInterface;
pub use ordinator_orchestrator_actor_traits::SystemSolutions;
// TODO [ ] 2025-07-02 add the other `<Actor>Interface`s here
pub use ordinator_orchestrator_actor_traits::TacticalInterface;
// TODO END
pub use ordinator_scheduling_environment::Asset;
use ordinator_scheduling_environment::SchedulingEnvironment;
pub use ordinator_scheduling_environment::time_environment::day::Day;
pub use ordinator_scheduling_environment::work_order::WorkOrderNumber;
use ordinator_scheduling_environment::work_order::WorkOrders;
pub use ordinator_scheduling_environment::work_order::operation::ActivityNumber;
pub use ordinator_scheduling_environment::worker_environment::availability::Availability;
pub use ordinator_scheduling_environment::worker_environment::resources::ActorCompositeId;
pub use ordinator_scheduling_environment::worker_environment::resources::Resources;
use ordinator_strategic_actor::StrategicApi;
use ordinator_strategic_actor::algorithm::strategic_solution::StrategicSolution;
pub use ordinator_strategic_actor::messages::StrategicRequestMessage;
pub use ordinator_strategic_actor::messages::StrategicResponseMessage;
use ordinator_supervisor_actor::SupervisorApi;
use ordinator_supervisor_actor::algorithm::supervisor_solution::SupervisorSolution;
pub use ordinator_supervisor_actor::messages::SupervisorRequestMessage;
pub use ordinator_supervisor_actor::messages::SupervisorResponseMessage;
pub use ordinator_supervisor_actor::messages::requests::SupervisorStatusMessage;
pub use ordinator_supervisor_actor::messages::responses::SupervisorResponseStatus;
use ordinator_tactical_actor::TacticalApi;
use ordinator_tactical_actor::algorithm::tactical_solution::TacticalSolution;
pub use ordinator_tactical_actor::messages::TacticalRequestMessage;
pub use ordinator_tactical_actor::messages::TacticalResponseMessage;
pub use ordinator_tactical_actor::messages::requests::TacticalStatusMessage;
// use ordinator_total_data_processing::excel_dumps::create_excel_dump;
use serde::Deserialize;
use serde::Serialize;
use tokio::task::JoinHandle;
use tracing::debug;
use tracing::info;
use tracing::instrument;

use self::actor_registry::ActorRegistry;
use self::database::DataBaseConnection;
use self::logging::LogHandles;

// O
pub struct Orchestrator<Ss>
{
    pub scheduling_environment: Arc<std::sync::Mutex<SchedulingEnvironment>>,
    pub system_solutions: std::sync::Mutex<HashMap<Asset, Arc<ArcSwap<Ss>>>>,
    pub actor_registries: std::sync::Mutex<HashMap<Asset, ActorRegistry>>,
    pub error_channels: (Sender<anyhow::Error>, Receiver<anyhow::Error>),
    pub state_link_bus: std::sync::Mutex<bus::Bus<StateLink>>,
    pub system_configurations: Arc<ArcSwap<SystemConfigurations>>,
    pub database_connections: DataBaseConnection,
    pub system_clock_tick_receiver: Receiver<DateTime<Utc>>,
    pub system_clock_time_commands_sender: Option<Sender<TimeCommand>>,
    pub log_handles: LogHandles,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum OrchestratorRequest
{
    GetWorkOrderStatus(WorkOrderNumber),
    GetWorkOrdersState(Asset),
    GetPeriods,
    GetDays,
    AgentStatusRequest,
    // InitializeSystemAgentsFromFile(Asset, ActorSpecifications),
    CreateSupervisorAgent(Asset, u64, ActorCompositeId),
    DeleteSupervisorAgent(Asset, String),

    // This should be an API handle not simply
    // CreateOperationalAgent(Asset, Id, f64, OperationalConfiguration),
    DeleteOperationalAgent(Asset, String),
    Export(Asset),
}

pub enum StartError
{
    AlreadyRunning,
    CouldNotConstruct,
    CouldNotCreateDependencies,
}

// These are basically handlers on the `Orchestrator` I think that they
// should go into the. You have learned so much here but you have to
// keep going. Remember to follow your guts here.
impl<Ss> Orchestrator<Ss>
where
    Ss: SystemSolutions<
            Strategic = StrategicSolution,
            Tactical = TacticalSolution,
            Supervisor = SupervisorSolution,
            Operational = OperationalSolution,
        > + Send
        + Sync,
{
    #[instrument(level = "info", skip_all)]
    pub async fn handle(
        &self,
        orchestrator_request: OrchestratorRequest,
    ) -> Result<OrchestratorResponse>
    {
        match orchestrator_request {
            OrchestratorRequest::AgentStatusRequest => {
                // for asset in self.agent_registries.keys() {
                //     let strategic_agent_addr = &self
                //         .agent_registries
                //         .get(asset)
                //         .unwrap()
                //         .strategic_agent_sender;

                //     let tactical_agent_addr = &self
                //         .agent_registries
                //         .get(asset)
                //         .unwrap()
                //         .tactical_agent_sender;

                //     // What should we do here? I think that the best approach will be to make
                // the     // code function
                //     strategic_agent_addr.sender.send(ActorMessage::Actor(
                //         StrategicRequestMessage::Status(StrategicStatusMessage::General),
                //     ))?;

                //     tactical_agent_addr.sender.send(ActorMessage::Actor(
                //         TacticalRequestMessage::Status(TacticalStatusMessage::General),
                //     ))?;

                //     for (_id, addr) in self
                //         .agent_registries
                //         .get(asset)
                //         .unwrap()
                //         .supervisor_agent_senders
                //         .iter()
                //     {
                //         addr.sender
                //             .send(ActorMessage::Actor(SupervisorRequestMessage::Status(
                //                 SupervisorStatusMessage::General,
                //             )))?
                //     }

                //     for (_id, addr) in self
                //         .agent_registries
                //         .get(asset)
                //         .unwrap()
                //         .operational_agent_senders
                //         .iter()
                //     {
                //         addr.sender.send(ActorMessage::Actor(
                //
                // OperationalRequestMessage::Status(OperationalStatusRequest::General),
                //         ))?;
                //     }

                //     let agent_status = self
                //         .agent_registries
                //         .get(asset)
                //         .expect("Asset should always be present")
                //         .recv_all_agents_status()?;

                //     agent_status_by_asset.insert(asset.clone(), agent_status);
                // }
                // let orchestrator_response_status =
                // AgentStatusResponse::new(agent_status_by_asset);
                // let orchestrator_response =
                //     OrchestratorResponse::AgentStatus(orchestrator_response_status);
                Ok(OrchestratorResponse::Success)
            }
            // Do we want to use this? No.. Or actually yes.. We want to use the...
            // We want to use either the SystemConfiguration, or the ActorEnvironment here. I think
            // that is really the crux of the issue here.
            // QUESTION [ ]
            // How to make the code function correctly with the code here? I think that the best
            // thing to do here is put the actor specification in as part of the database. Why are
            // you hesitant? I am hesitant as I do not know the extend of the issue here. The best
            // thing to do is to make the code run with the, this means that the data should be
            // loaded from the `database` and not simply be a configuration. I think that means
            // that the seperate... You could save a lot of code by making the mongodb at the
            // center of all this... No I think that it is better. Remember that the code should
            // work correctly with the database and the with the.
            //
            // So what is the dataflow here? You
            // You should move the code into the SchedulingEnvironment. The TotalSap should handle
            // the initialization
            OrchestratorRequest::GetWorkOrderStatus(work_order_number) => {
                let scheduling_environment_guard = self.scheduling_environment.lock().unwrap();

                let cloned_work_orders: &WorkOrders = &scheduling_environment_guard.work_orders;

                let work_order = cloned_work_orders
                    .inner
                    .get(&work_order_number)
                    .with_context(|| {
                        format!("{work_order_number:?} is not part of the SchedulingEnvironment")
                    })?;

                let asset = &work_order.functional_location().asset;
                let _work_order_view = work_order.view();

                let _api_solution = match self.system_solutions.lock().unwrap().get(asset) {
                    Some(arc_swap_shared_solution) => (arc_swap_shared_solution).load(),
                    None => bail!("Asset: {:?} is not initialzed", &asset),
                };

                // let work_order_response = WorkOrderResponse::new(
                //     work_order,
                //     (**api_solution).clone().into(),
                //     work_order_configurations,
                // );
                bail!("Implement this")
            }
            OrchestratorRequest::GetWorkOrdersState(asset) => {
                let scheduling_environment_guard = self.scheduling_environment.lock().unwrap();

                let cloned_work_orders: &WorkOrders = &scheduling_environment_guard.work_orders;
                // This is not the correct implementation.
                let _work_orders: Vec<_> = cloned_work_orders
                    .inner
                    .iter()
                    .filter(|wo| wo.1.functional_location().asset == asset)
                    .collect();

                let _loaded_shared_solution =
                    match self.system_solutions.lock().unwrap().get(&asset) {
                        Some(arc_swap_shared_solution) => arc_swap_shared_solution.load(),
                        None => bail!("Ordinator has not been initialized for asset: {}", &asset),
                    };

                // let work_order_configurations = &work_orders.work_order_configurations;
                // let work_order_responses: HashMap<WorkOrderNumber, WorkOrderResponse> =
                // work_orders     .inner
                //     .iter()
                //     .map(|(work_order_number, work_order)| {
                //         let work_order_response = WorkOrderResponse::new(
                //             work_order,
                //             (**loaded_shared_solution).clone().into(),
                //             work_order_configurations,
                //         );
                //         (*work_order_number, work_order_response)
                //     })
                //     .collect();

                bail!("Implement this");
            }
            OrchestratorRequest::GetPeriods => {
                let scheduling_environment_guard = self.scheduling_environment.lock().unwrap();

                let periods = scheduling_environment_guard
                    .time_environment
                    .periods
                    .clone();

                let strategic_periods = OrchestratorResponse::Periods(periods);
                Ok(strategic_periods)
            }
            OrchestratorRequest::GetDays => {
                let scheduling_environment_guard = self.scheduling_environment.lock().unwrap();

                let days = scheduling_environment_guard.time_environment.days.clone();

                let tactical_days = OrchestratorResponse::Days(days);
                Ok(tactical_days)
            }
            OrchestratorRequest::CreateSupervisorAgent(
                _asset,
                _number_of_supervisor_periods,
                _id_string,
            ) => {
                // FIX
                // Here you should create the system so that an entry in the
                // `SchedulingEnvironment` is created.
                // todo!();
                // FIX
                // The methods should be defined on the `actor_factory`
                // This should be encapsulated. The factory method and the registry should be of
                // the same process. Should this be inside of the `Orchestrator`
                // or the `ActorFactory`? I think that the. So where should this
                // be defined. I think that the best component is the Orchestrator itself.
                // TODO [x] Make trait
                // TODO [ ] Make method on Orchestrator
                // TODO [ ] Integrate `ActorRegistry`
                //
                // FIX [ ] Make a `self.start_supervisor`

                // let orchestrator_response =
                // OrchestratorResponse::RequestStatus(response_string);

                Ok(OrchestratorResponse::Todo)
            }
            OrchestratorRequest::DeleteSupervisorAgent(asset, id_string) => {
                let id = self
                    .actor_registries
                    .lock()
                    .unwrap()
                    .get(&asset)
                    .unwrap()
                    .supervisor_by_id_string(id_string);

                self.actor_registries
                    .lock()
                    .unwrap()
                    .get_mut(&asset)
                    .unwrap()
                    .supervisor_agent_senders
                    .remove(&id);

                let response_string = format!("Supervisor agent deleted with id {id}");
                let orchestrator_response = OrchestratorResponse::RequestStatus(response_string);
                Ok(orchestrator_response)
            }
            // Do we even want this?
            // Yes it is crucial that `OperationalAgent`s can be created on demand. There is no
            // excuse for not having that function.
            // OrchestratorRequest::CreateOperationalAgent(
            //     asset,
            //     id,
            //     hours_per_day,
            //     operational_configuration,
            // ) => {
            //     // This function should update the scheduling environment and then create
            //     // a function should be called on the scheduling environment to process the
            //     // requests to create an agent.
            //     // FIX
            //     // QUESTION
            //     // What should this function do?
            //     // It creates an `OperationalAgent` but that is not enough.
            //     let response_string = format!("Operational agent created with id {}", id);

            //     let operational_configuration_all = OperationalConfigurationAll::new(
            //         id.clone(),
            //         hours_per_day,
            //         operational_configuration,
            //     );

            //     // WARN
            //     // You should create this so that the whole system is optimized
            //     // you should create the configuration. Let `create_operational_agent`
            //     // borrow it. And then insert it into the `SchedulingEnvironment`.
            //     self.create_operational_agent(&operational_configuration_all)?;
            //     // WARN
            //     // Is this API fault tolerant enough? I am not really sure.
            //     self.scheduling_environment
            //         .lock()
            //         .unwrap()
            //         .worker_environment
            //         .agent_environment
            //         .operational
            //         .insert(id, operational_configuration_all);

            //     let orchestrator_response = OrchestratorResponse::RequestStatus(response_string);

            //     Ok(orchestrator_response)
            // }
            OrchestratorRequest::DeleteOperationalAgent(asset, id_string) => {
                let id = self
                    .actor_registries
                    .lock()
                    .unwrap()
                    .get(&asset)
                    .unwrap()
                    .supervisor_by_id_string(id_string.clone());

                self.actor_registries
                    .lock()
                    .unwrap()
                    .get_mut(&asset)
                    .unwrap()
                    .operational_agent_senders
                    .remove(&id);

                let response_string = format!("Operational agent deleted  with id {id_string}");
                let orchestrator_response = OrchestratorResponse::RequestStatus(response_string);
                Ok(orchestrator_response)
            }
            OrchestratorRequest::Export(_asset) => {
                panic!();
            }
        }
    }

    // QUESTION
    // Is it correct to remove the agents here? I believe yes, the system have the
    // agents that it does. In the scheduling environment. I do not think that
    // we should move too much with this.
    // TODO [ ]
    // This should be a part of the asset_builder. Yes that is the correct way of
    // going about it.
    // Do not make a complete builder.
    // FIX You should simply delete this message.
}

// You need to decouple the messages from the crates. How should
// that be done? You need to create a trait with the correct kinds
// of... God what is the right path forward here? You should make
// tie them together here. I think that it the best approach.
//
// The idea is that you have a single function and then you decide to
// make this function correctly with the right kind of
//
// You had completely misunderstood how this should work. Great that you are
// growing so fast!
impl ActorRegistry
{
    fn new(
        strategic_agent_addr: Communication<StrategicRequestMessage, StrategicResponseMessage>,
        tactical_agent_addr: Communication<TacticalRequestMessage, TacticalResponseMessage>,
        supervisor_agent_addrs: HashMap<
            ActorCompositeId,
            Communication<SupervisorRequestMessage, SupervisorResponseMessage>,
        >,
        operational_actor_communication: HashMap<
            ActorCompositeId,
            Communication<OperationalRequestMessage, OperationalResponseMessage>,
        >,
    ) -> Self
    {
        ActorRegistry {
            strategic_agent_sender: strategic_agent_addr,
            tactical_agent_sender: tactical_agent_addr,
            supervisor_agent_senders: supervisor_agent_addrs,
            operational_agent_senders: operational_actor_communication,
        }
    }

    pub fn add_supervisor_agent(
        &mut self,
        id: ActorCompositeId,
        communication: Communication<SupervisorRequestMessage, SupervisorResponseMessage>,
    )
    {
        self.supervisor_agent_senders.insert(id, communication);
    }

    pub fn add_operational_agent(
        &mut self,
        id: ActorCompositeId,
        communication: Communication<OperationalRequestMessage, OperationalResponseMessage>,
    )
    {
        self.operational_agent_senders.insert(id, communication);
    }

    pub fn supervisor_by_id_string(&self, id_string: String) -> ActorCompositeId
    {
        self.supervisor_agent_senders
            .keys()
            .find(|id| id.0 == id_string)
            .unwrap()
            .clone()
    }
}

pub type OrchestratorBuildOutput<Ss> = Result<(
    Arc<Orchestrator<Ss>>,
    JoinHandle<Result<()>>,
    JoinHandle<()>,
)>;

pub enum Environment
{
    Prod,
    Test(DateTime<Utc>),
}

// This should be removed and replaced with a dyn
impl<Ss> Orchestrator<Ss>
where
    Ss: SystemSolutions<
            Strategic = StrategicSolution,
            Tactical = TacticalSolution,
            Supervisor = SupervisorSolution,
            Operational = OperationalSolution,
        >
        + Send
        + Sync
        + 'static
        + Debug,
{
    pub fn builder() -> OrchestratorBuilder<StepLogging>
    {
        OrchestratorBuilder::<StepLogging> {
            logging: None,
            system_clock_tick_receiver: None,
            system_clock_time_commands_sender: None,
            system_clock_handle: None,
            system_configurations: None,
            scheduling_environment: None,
            _marker: PhantomData::<StepLogging>,
        }
    }

    // This is made in a wrong way. You should put the code into the
    // What should be done here? You need to provide the Orchestrator with
    // a `SchedulingEnvironment` so that you can test it. At the moment the
    //
    // scheduling environment can only be supplied through files.
    // Make the builder afterwards. Now you have to focus on the
    async fn actor_error_handler(error_receiver: Receiver<anyhow::Error>) -> Result<()>
    {
        // This function will become important if [`ActorError`]s should
        // not simply crash the Actors
        // loop {
        match error_receiver.recv_async().await {
            // This is the actor error
            Ok(actor_error) => Err(actor_error),
            Err(_) => Err(anyhow!("All actors are down")),
        }
        // }
    }

    pub fn asset_factory(&self, asset: &Asset) -> Result<&Self>
    {
        // WARN: DO NOT CHANGE THE "0" HERE. It is forcing the Orchestrator to handle
        // Actor errors before the Actor(s) can continue running.

        let system_solution = Arc::new(ArcSwap::new(Arc::new(Ss::new())));

        self.system_solutions
            .lock()
            .unwrap()
            .insert(asset.clone(), system_solution);
        let dependencies = self.extract_factory_dependencies(asset)?;

        let (strategic_id, tactical_id, supervisors, operationals) = {
            let scheduling_environment_guard = self.scheduling_environment.lock().unwrap();
            let actor_specifications = scheduling_environment_guard
                .worker_environment
                .actor_specification
                .get(asset)
                .unwrap();
            let periods = &scheduling_environment_guard.time_environment.periods;
            let days = &scheduling_environment_guard.time_environment.days;

            let input_strategic = &actor_specifications.strategic();

            let strategic_id = ActorCompositeId::new(
                &input_strategic.id,
                vec![],
                Availability::new(
                    *periods.first().unwrap().start_datetime(),
                    *periods
                        .get(input_strategic.number_of_strategic_periods - 1)
                        .or_else(|| periods.last())
                        .expect("Time not initialized correctly")
                        .finish_datetime(),
                    vec![asset.clone()],
                )?,
            );

            // ISSUE: #000 [ ] - Make the `TimeEnvironment` return function based times
            // ESSAY: #20250814-1
            // You should make the time_environment into a system clock. No that
            // fits into a port, and the adapter will then handle the relehj
            let input_tactical = &actor_specifications.tactical();
            debug!(target: "developer", days = ?days);
            let tactical_id = ActorCompositeId::new(
                &actor_specifications.tactical().id,
                vec![],
                Availability::new(
                    days.first()
                        .unwrap()
                        .date
                        .and_hms_opt(0, 0, 0)
                        .context("Could not make a DateTime in Availability for TacticalActor")?
                        .and_utc(),
                    days.get(input_tactical.number_of_tactical_days - 1)
                        .or_else(|| days.last())
                        .unwrap()
                        .date
                        .and_hms_opt(0, 0, 0)
                        .context("Could not make a DateTime in Availability for TacticalActor")?
                        .and_utc(),
                    vec![asset.clone()],
                )?,
            );

            let supervisors = actor_specifications.supervisor();

            let mut supervisor_ids: Vec<ActorCompositeId> = vec![];
            for input_supervisor in supervisors {
                let supervisor_actor_id = ActorCompositeId::new(
                    &input_supervisor.id,
                    vec![],
                    Availability::new(
                        *periods.first().unwrap().start_datetime(),
                        *periods
                            .get((input_supervisor.number_of_supervisor_periods - 1) as usize)
                            .unwrap()
                            .finish_datetime(),
                        vec![asset.clone()],
                    )?,
                );

                supervisor_ids.push(supervisor_actor_id)
            }

            let operationals: Vec<ActorCompositeId> = actor_specifications
                .operational()
                .iter()
                // TODO [ ] Start here.
                // Loop over all the availabilities in the system using `InputOperational`.
                .flat_map(|(id, input_operational)| {
                    input_operational
                        .operational_configuration
                        .availability
                        .iter()
                        .map(|availability| {
                            ActorCompositeId::new(
                                id,
                                input_operational
                                    .operational_configuration
                                    .resources
                                    .clone()
                                    .into_iter()
                                    .collect::<Vec<_>>(),
                                availability.clone(),
                            )
                        })
                })
                .collect::<Vec<_>>();

            (strategic_id, tactical_id, supervisor_ids, operationals)
        };

        let strategic_communication = StrategicApi::construct_actor(
            // You should make the code work correctly with the
            strategic_id.clone(),
            dependencies.0.clone(),
            dependencies.1.clone(),
            dependencies.2.clone(),
            self.state_link_bus.lock().unwrap().add_rx(),
            self.error_channels.0.clone(),
        )
        .with_context(|| format!("Could not construct StartegicActor {strategic_id}"))?;

        // Where should their IDs come from? I think that the best approach is to
        // include them from

        let tactical_communication = TacticalApi::construct_actor(
            tactical_id.clone(),
            dependencies.0.clone(),
            dependencies.1.clone(),
            dependencies.2.clone(),
            self.state_link_bus.lock().unwrap().add_rx(),
            self.error_channels.0.clone(),
        )
        .with_context(|| format!("{tactical_id} could not be constructed"))?;

        // // This is a good sign. It means that the system is performing correctly.
        // What // should be done about the code in general?
        // // Why is the supervisor no used here? This is also not created in the best
        // way.

        let mut supervisor_communications = HashMap::default();
        for supervisor_id in supervisors {
            let supervisor_communication = SupervisorApi::construct_actor(
                supervisor_id.clone(),
                dependencies.0.clone(),
                dependencies.1.clone(),
                dependencies.2.clone(),
                self.state_link_bus.lock().unwrap().add_rx(),
                self.error_channels.0.clone(),
            )?;

            supervisor_communications.insert(supervisor_id.clone(), supervisor_communication);
        }

        let mut operational_communications = HashMap::default();
        for operational_id in operationals {
            let operational_communication = OperationalApi::construct_actor(
                operational_id.clone(),
                dependencies.0.clone(),
                dependencies.1.clone(),
                dependencies.2.clone(),
                self.state_link_bus.lock().unwrap().add_rx(),
                self.error_channels.0.clone(),
            )?;

            operational_communications.insert(operational_id.clone(), operational_communication);
        }

        // The flexibility of making a `HashMap` is a good idea.
        let agent_registry = ActorRegistry::new(
            strategic_communication,
            tactical_communication,
            supervisor_communications,
            operational_communications,
        );

        self.actor_registries
            .lock()
            .unwrap()
            .insert(asset.clone(), agent_registry);
        info!(target: "stdout", "System initialized (3 of 4): Asset {}", asset);

        Ok(self)
    }
}

pub struct StepLogging;
pub struct StepSystemClock;
pub struct StepConfiguration;
pub struct StepSchedulingEnvironment;
pub struct StepBuild;

// pub struct Orchestrator<Ss>
// {
//     pub scheduling_environment: Arc<std::sync::Mutex<SchedulingEnvironment>>,
//     pub system_solutions: std::sync::Mutex<HashMap<Asset, Arc<ArcSwap<Ss>>>>,
//     pub actor_registries: std::sync::Mutex<HashMap<Asset, ActorRegistry>>,
//     pub error_channels: (Sender<anyhow::Error>, Receiver<anyhow::Error>),
//     pub state_link_bus: std::sync::Mutex<bus::Bus<StateLink>>,
//     pub system_configurations: Arc<ArcSwap<SystemConfigurations>>,
//     pub database_connections: DataBaseConnection,
//     pub system_clock_tick_receiver: Receiver<DateTime<Utc>>,
//     pub system_clock_time_commands_sender: Option<Sender<TimeCommand>>,
//     pub log_handles: LogHandles,
// }

pub struct OrchestratorBuilder<Step>
{
    logging: Option<LogHandles>,
    system_clock_tick_receiver: Option<Receiver<DateTime<Utc>>>,
    system_clock_time_commands_sender: Option<Option<Sender<TimeCommand>>>,
    system_clock_handle: Option<JoinHandle<()>>,
    system_configurations: Option<Arc<ArcSwap<SystemConfigurations>>>,
    scheduling_environment: Option<Arc<std::sync::Mutex<SchedulingEnvironment>>>,
    _marker: PhantomData<Step>,
}

impl OrchestratorBuilder<StepLogging>
{
    pub fn logging(self, logging: LogHandles) -> OrchestratorBuilder<StepSystemClock>
    {
        OrchestratorBuilder::<StepSystemClock> {
            logging: Some(logging),
            system_clock_tick_receiver: None,
            system_clock_time_commands_sender: None,
            system_clock_handle: None,
            system_configurations: None,
            scheduling_environment: None,
            _marker: PhantomData,
        }
    }
}
impl OrchestratorBuilder<StepSystemClock>
{
    pub fn system_clock(self, environment: &Environment) -> OrchestratorBuilder<StepConfiguration>
    {
        let (system_clock_handle, system_clock_tick_receiver, system_clock_time_commands_sender): (
            JoinHandle<()>,
            Receiver<DateTime<Utc>>,
            Option<Sender<TimeCommand>>,
        ) = match environment {
            Environment::Test(current_time) => {
                let (system_clock_time_commands_sender, system_clock_time_commands_receiver) =
                    flume::unbounded();
                let (system_clock_tick_sender, system_clock_tick_receiver) = flume::unbounded();

                let system_clock_handle = TestSystemClock::new(
                    *current_time,
                    system_clock_time_commands_receiver,
                    system_clock_tick_sender,
                )
                .start_system_clock();

                (
                    system_clock_handle,
                    system_clock_tick_receiver,
                    Some(system_clock_time_commands_sender),
                )
            }
            Environment::Prod => {
                let (system_clock_tick_sender, system_clock_tick_receiver) = flume::unbounded();

                let system_clock_handle =
                    ProductionSystemClock::new(system_clock_tick_sender).start_system_clock();

                (system_clock_handle, system_clock_tick_receiver, None)
            }
        };

        OrchestratorBuilder::<StepConfiguration> {
            logging: self.logging,
            system_clock_tick_receiver: Some(system_clock_tick_receiver),
            system_clock_time_commands_sender: Some(system_clock_time_commands_sender),
            system_clock_handle: Some(system_clock_handle),

            system_configurations: None,
            scheduling_environment: None,
            _marker: PhantomData,
        }
    }
}

// The most important thing is that you can sustain a good pace all the time
// moving forward continuously and improving.
impl OrchestratorBuilder<StepConfiguration>
{
    pub fn system_configurations(self) -> OrchestratorBuilder<StepSchedulingEnvironment>
    {
        let configurations = SystemConfigurations::read_all_configs().unwrap();
        OrchestratorBuilder::<StepSchedulingEnvironment> {
            logging: self.logging,
            system_clock_tick_receiver: self.system_clock_tick_receiver,
            system_clock_time_commands_sender: self.system_clock_time_commands_sender,
            system_clock_handle: self.system_clock_handle,
            system_configurations: Some(configurations),
            scheduling_environment: None,
            _marker: PhantomData,
        }
    }
}

impl OrchestratorBuilder<StepSchedulingEnvironment>
{
    pub fn scheduling_environment_from_database(
        self,
        asset: &Asset,
    ) -> anyhow::Result<OrchestratorBuilder<StepBuild>>
    {
        let system_clock_at_initialization = self
            .system_clock_tick_receiver
            .as_ref()
            .expect("previous builder created this")
            .recv()
            .expect("Previous builder made this");

        let system_configurations = self
            .system_configurations
            .expect("Previous builder created this")
            .clone();

        // ISSUE #000 - Asset should not be specified here.
        let scheduling_environment = DataBaseConnection::scheduling_environment(
            system_clock_at_initialization,
            asset.clone(),
            system_configurations.clone(),
        )
        .context("Could not build SchedulingEnvironment")?;

        Ok(OrchestratorBuilder::<StepBuild> {
            logging: self.logging,
            system_clock_tick_receiver: self.system_clock_tick_receiver,
            system_clock_time_commands_sender: self.system_clock_time_commands_sender,
            system_clock_handle: self.system_clock_handle,
            system_configurations: Some(system_configurations),
            scheduling_environment: Some(scheduling_environment),
            _marker: PhantomData,
        })
    }

    pub fn scheduling_environment_manual(
        self,
        scheduling_environment: Arc<std::sync::Mutex<SchedulingEnvironment>>,
    ) -> OrchestratorBuilder<StepBuild>
    {
        OrchestratorBuilder::<StepBuild> {
            logging: self.logging,
            system_clock_tick_receiver: self.system_clock_tick_receiver,
            system_clock_time_commands_sender: self.system_clock_time_commands_sender,
            system_clock_handle: self.system_clock_handle,
            system_configurations: self.system_configurations,
            scheduling_environment: Some(scheduling_environment),
            _marker: PhantomData,
        }
    }
}

impl OrchestratorBuilder<StepBuild>
{
    pub fn build<Ss>(self) -> OrchestratorBuildOutput<Ss>
    where
        Ss: SystemSolutions<
                Strategic = StrategicSolution,
                Tactical = TacticalSolution,
                Supervisor = SupervisorSolution,
                Operational = OperationalSolution,
            >
            + Send
            + Sync
            + 'static
            + Debug,
    {
        let error_channels: (Sender<anyhow::Error>, Receiver<anyhow::Error>) = flume::bounded(0);

        let error_task_handle: JoinHandle<Result<()>> = tokio::spawn(
            Orchestrator::<Ss>::actor_error_handler(error_channels.1.clone()),
        );

        // WARN THIS SHOULD BE CHANGED
        // The primary issue here is that the code is not made for testing. That is a
        // huge issue, you should ideally inject a test time only to test the
        // components that need this. You are making something that is not the best
        // approach. Also, you should refactor all

        // This should be a bus::Bus instead.
        // The current time should come from the SystemClock not the other way
        // around. You are experiencing pain. And that pain is what is needed to
        // grow and solve this problem.
        // This is completely wrong... Not you simply have to be able to inject your own
        // scheduling environment here. That is the main issue with this
        // function.

        // CRUCIAL LESSON: For types that there exists multiple versions of always
        // qualify the whole path.
        let state_link_bus: std::sync::Mutex<bus::Bus<StateLink>> =
            std::sync::Mutex::new(bus::Bus::new(5));
        // WARN THIS SHOULD BE CHANGED
        let orchestrator = Orchestrator::<Ss> {
            scheduling_environment: self.scheduling_environment.expect("Should be type safe"),
            system_solutions: std::sync::Mutex::new(HashMap::new()),
            actor_registries: std::sync::Mutex::new(HashMap::new()),
            error_channels,
            state_link_bus,
            system_configurations: self.system_configurations.expect("Should be type safe"),
            // We are not using it yet and you should remove it from the system
            database_connections: DataBaseConnection,
            system_clock_tick_receiver: self
                .system_clock_tick_receiver
                .expect("Should be type safe"),
            system_clock_time_commands_sender: self
                .system_clock_time_commands_sender
                .expect("Should be type safe"),
            log_handles: self.logging.expect("Should be type safe"),
        };
        info!(target: "stdout", "System initialized (2 of 4): orchestrator");
        Ok((
            Arc::new(orchestrator),
            error_task_handle,
            self.system_clock_handle.expect("Should be type safe"),
        ))
    }
}

// let current_time = system_clock_tick_receiver.recv()?;

// fn start_steel_repl(arc_orchestrator: ArcOrchestrator) {
//     thread::spawn(move || {
// let mut steel_engine = steel::steel_vm::engine::Engine::new();
// steel_engine.register_type::<ArcOrchestrator>("Orchestrator?");
// steel_engine.register_fn("actor_registry",
// ArcOrchestrator::print_actor_registry); steel_engine.register_type::<Asset>("
// Asset?"); steel_engine.register_fn("Asset", Asset::new_from_string);

// steel_engine.register_external_value("asset::df", Asset::DF);
// steel_engine
//     .register_external_value("orchestrator", arc_orchestrator)
//     .unwrap();

// steel_repl::run_repl(steel_engine).unwrap();
//     });
// }
impl<Ss> Orchestrator<Ss>
where
    Ss: SystemSolutions,
{
    pub fn export_xlsx_solution(&self, _asset: Asset) -> Result<(Vec<u8>, String)>
    {
        // let system_solution = self
        //     .system_solutions
        //     .get(&asset)
        //     .with_context(|| {
        //         format!("Could not retrieve the shared_solution for asset
        // {asset:#?}")     })?
        //     .load();

        // This is where it gets a little weird. The handlers should only call methods
        // on the orchestrator.
        // This function should lie in the `orchestrator` crate. How in the world did it
        // ever end up in here
        // let strategic_agent_solution =
        // system_solution.strategic().all_scheduled_tasks();
        // let tactical_agent_solution =
        // system_solution.tactical().all_scheduled_tasks();
        let _work_orders = {
            let scheduling_environment_lock = self.scheduling_environment.lock().unwrap();
            scheduling_environment_lock.work_orders.clone()
        };

        // ISSUE #000 [ ] - introduce the `create_excel_dump` in the system.
        // let xlsx_filename = create_excel_dump(
        //     asset.clone(),
        //     work_orders,
        //     self.system_solutions
        //         .lock()
        //         .unwrap()
        //         .get(&asset)
        //         .with_context(|| {
        //             format!("You should start up a Scheduling System for Asset
        // {asset}")         })?
        //         .load(),
        // )
        // .unwrap();
        // let mut buffer = Vec::new();
        // let mut file = File::open(&xlsx_filename).unwrap();
        // file.read_to_end(&mut buffer).unwrap();
        // std::fs::remove_file(xlsx_filename).expect("The XLSX file could not be
        // deleted"); let filename = format!("ordinator_xlsx_dump_for_{asset}");
        // let http_header = format!("attachment; filename={filename}");

        // Ok((buffer, http_header))
        Err(anyhow!("REIMPLEMENT THE EXCEL EXPORT FUNCTION"))
    }
}

// We should start the system clock and then have it send messages to the
// Orchestrator. That means that the Orchestrator should have a future the
// same as the `error_channel`. Yes that is the best approach here. I do
//
pub enum TimeCommand
{
    Advance(chrono::Duration),
    SetTime(chrono::DateTime<Utc>),
}
// ISSUE #000 TODO [ ] 2025-07-21 fix the message channel. Make the
// [`Orchestrator`] await the [`SystemClock`].
pub struct TestSystemClock
{
    current_time: chrono::DateTime<chrono::Utc>,
    system_clock_time_commands: flume::Receiver<TimeCommand>,
    system_clock_tick: flume::Sender<DateTime<Utc>>,
}

impl TestSystemClock
{
    pub fn new(
        current_time: chrono::DateTime<chrono::Utc>,
        system_clock_time_commands: flume::Receiver<TimeCommand>,
        system_clock_tick: flume::Sender<DateTime<Utc>>,
    ) -> Self
    {
        Self {
            current_time,
            system_clock_time_commands,
            system_clock_tick,
        }
    }
}

pub trait SystemClock
{
    fn now(&self) -> chrono::DateTime<Utc>;
    fn start_system_clock(self) -> JoinHandle<()>;
}

pub struct ProductionSystemClock
{
    system_clock_tick: flume::Sender<DateTime<Utc>>,
}

impl ProductionSystemClock
{
    pub fn new(system_clock_tick: flume::Sender<DateTime<Utc>>) -> Self
    {
        Self { system_clock_tick }
    }
}

// Okay, just quickly get this working. I think that the best approach here
// is to make the system work quickly first, and then know that you have a
// place for all time related logic in here.
impl SystemClock for TestSystemClock
{
    fn now(&self) -> chrono::DateTime<Utc>
    {
        self.current_time
    }

    fn start_system_clock(mut self) -> JoinHandle<()>
    {
        tokio::spawn(async move {
            loop {
                while let Ok(command) = self.system_clock_time_commands.try_recv() {
                    match command {
                        TimeCommand::Advance(time_delta) => self.current_time += time_delta,
                        TimeCommand::SetTime(date_time) => self.current_time = date_time,
                    }
                }
                let current_datetime = self.current_time;
                self.system_clock_tick.send(current_datetime).unwrap();
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        })
    }
}
// The ProductionClock should only have the ticker. You are not allowed to
// modify the timer. Is this correct? Yes I think so. You are making good
// progress here.
impl SystemClock for ProductionSystemClock
{
    fn now(&self) -> chrono::DateTime<Utc>
    {
        chrono::Utc::now()
    }

    fn start_system_clock(self) -> JoinHandle<()>
    {
        tokio::spawn(async move {
            loop {
                let current_datetime = Utc::now();
                self.system_clock_tick.send(current_datetime).unwrap();
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        })
    }
}
