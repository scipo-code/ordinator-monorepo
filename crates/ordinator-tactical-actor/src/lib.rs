pub mod algorithm;
pub mod messages;

use std::fmt::Debug;
use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::Arc;
use std::sync::Mutex;

use algorithm::ProjectAlgorithm;
use algorithm::tactical_parameters::ProjectParameters;
use algorithm::tactical_solution::ProjectSolution;
use anyhow::Result;
use arc_swap::ArcSwap;
use bus::BusReader;
use flume::Sender;
use messages::ProjectRequestMessage;
use messages::ProjectResponseMessage;
use ordinator_actor_core::Actor;
use ordinator_actor_core::algorithm::Algorithm;
use ordinator_actor_core::traits::ActorBasedLargeNeighborhoodSearch;
use ordinator_configuration::SystemConfigurations;
use ordinator_orchestrator_actor_traits::ActorFactory;
use ordinator_orchestrator_actor_traits::CommandHandler;
use ordinator_orchestrator_actor_traits::Communication;
use ordinator_orchestrator_actor_traits::StateLink;
use ordinator_orchestrator_actor_traits::SystemSolutions;
use ordinator_scheduling_environment::SchedulingEnvironment;
use ordinator_scheduling_environment::work_order::WorkOrderNumber;
use ordinator_scheduling_environment::worker_environment::resources::ActorCompositeId;
use priority_queue::PriorityQueue;

pub struct ProjectActor<Ss: Debug>(
    Actor<ProjectRequestMessage, ProjectResponseMessage, ProjectAlgorithm<Ss>>,
)
where
    Ss: SystemSolutions<Project = ProjectSolution>,
    Actor<ProjectRequestMessage, ProjectResponseMessage, ProjectAlgorithm<Ss>>:
        CommandHandler<ProjectRequestMessage, ProjectResponseMessage>;

impl<Ss> Deref for ProjectActor<Ss>
where
    Ss: SystemSolutions<Project = ProjectSolution> + Debug,
    Actor<ProjectRequestMessage, ProjectResponseMessage, ProjectAlgorithm<Ss>>:
        CommandHandler<ProjectRequestMessage, ProjectResponseMessage>,
{
    type Target = Actor<ProjectRequestMessage, ProjectResponseMessage, ProjectAlgorithm<Ss>>;

    fn deref(&self) -> &Self::Target
    {
        &self.0
    }
}

impl<Ss: Debug> DerefMut for ProjectActor<Ss>
where
    Ss: SystemSolutions<Project = ProjectSolution>,
    Actor<ProjectRequestMessage, ProjectResponseMessage, ProjectAlgorithm<Ss>>:
        CommandHandler<ProjectRequestMessage, ProjectResponseMessage>,
{
    fn deref_mut(&mut self) -> &mut Self::Target
    {
        &mut self.0
    }
}

pub struct ProjectApi {}

impl<Ss> ActorFactory<Ss> for ProjectApi
where
    Ss: SystemSolutions<Project = ProjectSolution> + Send + Sync + 'static + Debug,
    ProjectAlgorithm<Ss>: ActorBasedLargeNeighborhoodSearch
        + Send
        + Sync
        + From<
            Algorithm<
                ProjectSolution,
                ProjectParameters,
                PriorityQueue<WorkOrderNumber, u64>,
                Ss,
            >,
        >,
    Actor<ProjectRequestMessage, ProjectResponseMessage, ProjectAlgorithm<Ss>>:
        CommandHandler<ProjectRequestMessage, ProjectResponseMessage>,
{
    type Communication = Communication<ProjectRequestMessage, ProjectResponseMessage>;

    fn construct_actor(
        id: ActorCompositeId,
        scheduling_environment_guard: Arc<Mutex<SchedulingEnvironment>>,
        shared_solution_arc_swap: Arc<ArcSwap<Ss>>,
        system_configurations: Arc<ArcSwap<SystemConfigurations>>,
        state_link_bus: BusReader<StateLink>,
        error_channel: Sender<anyhow::Error>,
    ) -> Result<Self::Communication>
    {
        Actor::<ProjectRequestMessage, ProjectResponseMessage, ProjectAlgorithm<Ss>>::builder()
            .agent_id(id.clone())
            .scheduling_environment(Arc::clone(&scheduling_environment_guard))
            // TODO: Refactor algorithm builder to simplify initialization
            .algorithm(|ab| {
                ab.id(id)
                    .parameters_and_solution(&scheduling_environment_guard.lock().unwrap())?
                    .system_solution_arc_swap(shared_solution_arc_swap)
            })?
            // TODO: Consolidate communication and state link creation
            .communication(error_channel, state_link_bus)
            .configurations(system_configurations)
            .build()
    }
}
