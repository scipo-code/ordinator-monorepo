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
use ordinator_configuration::throttling::Throttling;
use ordinator_contracts::orchestrator::OrchestratorResponse;
use ordinator_operational_actor::OperationalApi;
use ordinator_operational_actor::algorithm::operational_solution::OperationalSolution;
pub use ordinator_operational_actor::messages::OperationalRequestMessage;
pub use ordinator_operational_actor::messages::OperationalResponseMessage;
pub use ordinator_operational_actor::messages::requests::OperationalStatusRequest;
use ordinator_orchestrator_actor_traits::ActorFactory;
use ordinator_orchestrator_actor_traits::Communication;
pub use ordinator_orchestrator_actor_traits::StateLink;
pub use ordinator_orchestrator_actor_traits::WeeklyInterface;
pub use ordinator_orchestrator_actor_traits::SystemSolutions;
// TODO [ ] 2025-07-02 add the other `<Actor>Interface`s here
pub use ordinator_orchestrator_actor_traits::ProjectInterface;
// TODO END
pub use ordinator_scheduling_environment::Asset;
use ordinator_scheduling_environment::SchedulingEnvironment;
pub use ordinator_scheduling_environment::time_environment::day::Day;
pub use ordinator_scheduling_environment::work_order::WorkOrderNumber;
use ordinator_scheduling_environment::work_order::WorkOrders;
pub use ordinator_scheduling_environment::work_order::operation::ActivityNumber;
pub use ordinator_scheduling_environment::worker_environment::availability::Availability;
pub use ordinator_scheduling_environment::worker_environment::resources::ActorCompositeId;
pub use ordinator_scheduling_environment::worker_environment::resources::Skill;
use ordinator_strategic_actor::WeeklyApi;
use ordinator_strategic_actor::algorithm::strategic_solution::WeeklySolution;
pub use ordinator_strategic_actor::messages::WeeklyRequestMessage;
pub use ordinator_strategic_actor::messages::WeeklyResponseMessage;
use ordinator_supervisor_actor::DailyApi;
use ordinator_supervisor_actor::algorithm::supervisor_solution::DailySolution;
pub use ordinator_supervisor_actor::messages::DailyRequestMessage;
pub use ordinator_supervisor_actor::messages::DailyResponseMessage;
pub use ordinator_supervisor_actor::messages::requests::DailyStatusMessage;
pub use ordinator_supervisor_actor::messages::responses::DailyResponseStatus;
use ordinator_tactical_actor::ProjectApi;
use ordinator_tactical_actor::algorithm::tactical_solution::ProjectSolution;
pub use ordinator_tactical_actor::messages::ProjectRequestMessage;
pub use ordinator_tactical_actor::messages::ProjectResponseMessage;
pub use ordinator_tactical_actor::messages::requests::ProjectStatusMessage;
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

pub struct Orchestrator<Ss>
{
    pub scheduling_environment: Arc<std::sync::Mutex<SchedulingEnvironment>>,
    pub system_solutions: std::sync::Mutex<HashMap<Asset, Arc<ArcSwap<Ss>>>>,
    pub actor_registries: std::sync::Mutex<HashMap<Asset, ActorRegistry>>,
    pub error_sender: Sender<anyhow::Error>,
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
    CreateDailyAgent(Asset, u64, ActorCompositeId),
    DeleteDailyAgent(Asset, String),

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

impl<Ss> Orchestrator<Ss>
where
    Ss: SystemSolutions<
            Weekly = WeeklySolution,
            Project = ProjectSolution,
            Daily = DailySolution,
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
                Ok(OrchestratorResponse::Success)
            }
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
                // TODO: Implement work order filtering correctly
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
            OrchestratorRequest::CreateDailyAgent(
                _asset,
                _number_of_supervisor_periods,
                _id_string,
            ) => {
                // TODO: Implement supervisor agent creation
                Ok(OrchestratorResponse::Todo)
            }
            OrchestratorRequest::DeleteDailyAgent(asset, id_string) => {
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

                let response_string = format!("Daily agent deleted with id {id}");
                let orchestrator_response = OrchestratorResponse::RequestStatus(response_string);
                Ok(orchestrator_response)
            }
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

}

impl ActorRegistry
{
    fn new(
        strategic_agent_addr: Communication<WeeklyRequestMessage, WeeklyResponseMessage>,
        tactical_agent_addr: Communication<ProjectRequestMessage, ProjectResponseMessage>,
        supervisor_agent_addrs: HashMap<
            ActorCompositeId,
            Communication<DailyRequestMessage, DailyResponseMessage>,
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
        communication: Communication<DailyRequestMessage, DailyResponseMessage>,
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
    Receiver<anyhow::Error>,
    JoinHandle<()>,
)>;

#[derive(Clone)]
pub enum Environment
{
    Prod,
    Test(DateTime<Utc>),
}

impl<Ss> Orchestrator<Ss>
where
    Ss: SystemSolutions<
            Weekly = WeeklySolution,
            Project = ProjectSolution,
            Daily = DailySolution,
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

    pub fn asset_factory(&self, asset: &Asset) -> Result<&Self>
    {
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
                        .context("Could not make a DateTime in Availability for ProjectActor")?
                        .and_utc(),
                    days.get(input_tactical.number_of_tactical_days - 1)
                        .or_else(|| days.last())
                        .unwrap()
                        .date
                        .and_hms_opt(0, 0, 0)
                        .context("Could not make a DateTime in Availability for ProjectActor")?
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

        let strategic_communication = WeeklyApi::construct_actor(
            strategic_id.clone(),
            dependencies.0.clone(),
            dependencies.1.clone(),
            dependencies.2.clone(),
            self.state_link_bus.lock().unwrap().add_rx(),
            self.error_sender.clone(),
        )
        .with_context(|| format!("Could not construct StartegicActor {strategic_id}"))?;

        let tactical_communication = ProjectApi::construct_actor(
            tactical_id.clone(),
            dependencies.0.clone(),
            dependencies.1.clone(),
            dependencies.2.clone(),
            self.state_link_bus.lock().unwrap().add_rx(),
            self.error_sender.clone(),
        )
        .with_context(|| format!("{tactical_id} could not be constructed"))?;

        let mut supervisor_communications = HashMap::default();
        for supervisor_id in supervisors {
            let supervisor_communication = DailyApi::construct_actor(
                supervisor_id.clone(),
                dependencies.0.clone(),
                dependencies.1.clone(),
                dependencies.2.clone(),
                self.state_link_bus.lock().unwrap().add_rx(),
                self.error_sender.clone(),
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
                self.error_sender.clone(),
            )?;

            operational_communications.insert(operational_id.clone(), operational_communication);
        }

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

    pub fn system_configurations_manual(
        self,
        strategic_throttling: u64,
        tactical_throttling: u64,
        supervisor_throttling: u64,
        operational_throttling: u64,
    ) -> OrchestratorBuilder<StepSchedulingEnvironment>
    {
        let throttling = Throttling {
            strategic_throttling,
            tactical_throttling,
            supervisor_throttling,
            operational_throttling,
        };
        let configurations = SystemConfigurations::build_configs(throttling);
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

    pub fn system_configurations_defaults(self) -> OrchestratorBuilder<StepSchedulingEnvironment>
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
                Weekly = WeeklySolution,
                Project = ProjectSolution,
                Daily = DailySolution,
                Operational = OperationalSolution,
            >
            + Send
            + Sync
            + 'static
            + Debug,
    {
        let (sender, error_receiver): (Sender<anyhow::Error>, Receiver<anyhow::Error>) =
            flume::bounded(0);

        // TODO: Refactor to support dependency injection for testing
        let state_link_bus: std::sync::Mutex<bus::Bus<StateLink>> =
            std::sync::Mutex::new(bus::Bus::new(5));
        let orchestrator = Orchestrator::<Ss> {
            scheduling_environment: self.scheduling_environment.expect("Should be type safe"),
            system_solutions: std::sync::Mutex::new(HashMap::new()),
            actor_registries: std::sync::Mutex::new(HashMap::new()),
            state_link_bus,
            system_configurations: self.system_configurations.expect("Should be type safe"),
            database_connections: DataBaseConnection,
            system_clock_tick_receiver: self
                .system_clock_tick_receiver
                .expect("Should be type safe"),
            system_clock_time_commands_sender: self
                .system_clock_time_commands_sender
                .expect("Should be type safe"),
            log_handles: self.logging.expect("Should be type safe"),
            error_sender: sender,
        };
        info!(target: "stdout", "System initialized (2 of 4): orchestrator");
        Ok((
            Arc::new(orchestrator),
            error_receiver,
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
        let _work_orders = {
            let scheduling_environment_lock = self.scheduling_environment.lock().unwrap();
            scheduling_environment_lock.work_orders.clone()
        };

        Err(anyhow!("REIMPLEMENT THE EXCEL EXPORT FUNCTION"))
    }
}

pub enum TimeCommand
{
    Advance(chrono::Duration),
    SetTime(chrono::DateTime<Utc>),
}

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
