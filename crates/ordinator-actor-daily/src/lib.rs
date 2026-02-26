pub mod algorithm;
mod assert_functions;
pub mod messages;
use std::fmt::Debug;
use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::Arc;
use std::sync::Mutex;

use algorithm::DailyAlgorithm;
use algorithm::daily_parameters::DailyParameters;
use algorithm::daily_solution::DailySolution;
use anyhow::Result;
use arc_swap::ArcSwap;
#[allow(unused_imports)]
use assert_functions::DailyAssertions;
use bus::BusReader;
use flume::Sender;
use messages::DailyRequestMessage;
use messages::DailyResponseMessage;
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
use ordinator_scheduling_environment::worker_environment::resources::ActorCompositeId;
use ordinator_scheduling_hypergraph::schedule_graph::SchedulingHypergraph;

pub struct DailyActor<Ss: Debug>(
    Actor<DailyRequestMessage, DailyResponseMessage, DailyAlgorithm<Ss>>,
)
where
    Ss: SystemSolutions<Daily = DailySolution>,
    Actor<DailyRequestMessage, DailyResponseMessage, DailyAlgorithm<Ss>>:
        CommandHandler<DailyRequestMessage, DailyResponseMessage>;

impl<Ss> Deref for DailyActor<Ss>
where
    Ss: SystemSolutions<Daily = DailySolution> + Debug,
    Actor<DailyRequestMessage, DailyResponseMessage, DailyAlgorithm<Ss>>:
        CommandHandler<DailyRequestMessage, DailyResponseMessage>,
{
    type Target = Actor<DailyRequestMessage, DailyResponseMessage, DailyAlgorithm<Ss>>;

    fn deref(&self) -> &Self::Target
    {
        &self.0
    }
}

impl<Ss> DerefMut for DailyActor<Ss>
where
    Ss: SystemSolutions<Daily = DailySolution> + Debug,
    Actor<DailyRequestMessage, DailyResponseMessage, DailyAlgorithm<Ss>>:
        CommandHandler<DailyRequestMessage, DailyResponseMessage>,
{
    fn deref_mut(&mut self) -> &mut Self::Target
    {
        &mut self.0
    }
}

pub struct DailyApi {}

// Note: This implementation binds Ss type parameters to the impl block.
impl<Ss> ActorFactory<Ss> for DailyApi
where
    Ss: SystemSolutions<Daily = DailySolution> + Send + Sync + 'static + Debug,
    DailyAlgorithm<Ss>: ActorBasedLargeNeighborhoodSearch
        + Send
        + Sync
        + From<Algorithm<DailySolution, DailyParameters, (), Ss>>,
    Actor<DailyRequestMessage, DailyResponseMessage, DailyAlgorithm<Ss>>:
        CommandHandler<DailyRequestMessage, DailyResponseMessage>,
{
    type Communication = Communication<DailyRequestMessage, DailyResponseMessage>;

    fn construct_actor(
        id: ActorCompositeId,
        scheduling_environment_guard: Arc<Mutex<SchedulingHypergraph>>,
        shared_solution_arc_swap: Arc<ArcSwap<Ss>>,
        system_configurations: Arc<ArcSwap<SystemConfigurations>>,
        state_link_bus: BusReader<StateLink>,
        error_channel: Sender<anyhow::Error>,
    ) -> Result<Self::Communication>
    where
        Ss: SystemSolutions<Daily = DailySolution> + Send + Sync + 'static,
    {
        Actor::<DailyRequestMessage, DailyResponseMessage, DailyAlgorithm<Ss>>::builder()
            .agent_id(id.clone())
            .scheduling_environment(Arc::clone(&scheduling_environment_guard))
            .algorithm(|ab| {
                ab.id(id)
                    .parameters_and_solution(&scheduling_environment_guard.lock().unwrap())?
                    .system_solution_arc_swap(shared_solution_arc_swap)
            })?
            .communication(error_channel, state_link_bus)
            .configurations(system_configurations)
            .build()
    }
}
