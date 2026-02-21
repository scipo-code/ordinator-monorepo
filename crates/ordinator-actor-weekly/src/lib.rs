pub mod algorithm;
// pub mod assert_functions;
pub mod messages;

use std::fmt::Debug;
use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::Arc;
use std::sync::Mutex;

use algorithm::WeeklyAlgorithm;
use algorithm::weekly_parameters::WeeklyParameters;
use algorithm::weekly_solution::WeeklySolution;
use anyhow::Result;
use arc_swap::ArcSwap;
use bus::BusReader;
use flume::Sender;
use messages::WeeklyRequestMessage;
use messages::WeeklyResponseMessage;
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

pub struct WeeklyActor<Ss: Debug>(
    Actor<WeeklyRequestMessage, WeeklyResponseMessage, WeeklyAlgorithm<Ss>>,
)
where
    Ss: SystemSolutions<Weekly = WeeklySolution>,
    Actor<WeeklyRequestMessage, WeeklyResponseMessage, WeeklyAlgorithm<Ss>>:
        CommandHandler<WeeklyRequestMessage, WeeklyResponseMessage>;

impl<Ss> Deref for WeeklyActor<Ss>
where
    Ss: SystemSolutions<Weekly = WeeklySolution> + Debug,
    Actor<WeeklyRequestMessage, WeeklyResponseMessage, WeeklyAlgorithm<Ss>>:
        CommandHandler<WeeklyRequestMessage, WeeklyResponseMessage>,
{
    type Target = Actor<WeeklyRequestMessage, WeeklyResponseMessage, WeeklyAlgorithm<Ss>>;

    fn deref(&self) -> &Self::Target
    {
        &self.0
    }
}

impl<Ss> DerefMut for WeeklyActor<Ss>
where
    Ss: SystemSolutions<Weekly = WeeklySolution> + Debug,
    Actor<WeeklyRequestMessage, WeeklyResponseMessage, WeeklyAlgorithm<Ss>>:
        CommandHandler<WeeklyRequestMessage, WeeklyResponseMessage>,
{
    fn deref_mut(&mut self) -> &mut Self::Target
    {
        &mut self.0
    }
}

pub struct WeeklyApi {}
impl<Ss> ActorFactory<Ss> for WeeklyApi
where
    Ss: SystemSolutions<Weekly = WeeklySolution> + Send + Sync + 'static + Debug,
    Actor<WeeklyRequestMessage, WeeklyResponseMessage, WeeklyAlgorithm<Ss>>:
        CommandHandler<WeeklyRequestMessage, WeeklyResponseMessage>,
{
    type Communication = Communication<WeeklyRequestMessage, WeeklyResponseMessage>;

    fn construct_actor(
        id: ActorCompositeId,
        scheduling_environment_guard: Arc<Mutex<SchedulingEnvironment>>,
        shared_solution_arc_swap: Arc<ArcSwap<Ss>>,
        system_configurations: Arc<ArcSwap<SystemConfigurations>>,
        state_link_bus: BusReader<StateLink>,
        error_channel: Sender<anyhow::Error>,
    ) -> Result<<Self as ActorFactory<Ss>>::Communication>
    where
        Ss: SystemSolutions<Weekly = WeeklySolution> + Send + Sync + 'static,
        WeeklyAlgorithm<Ss>: ActorBasedLargeNeighborhoodSearch
            + Send
            + Sync
            + From<
                Algorithm<
                    WeeklySolution,
                    WeeklyParameters,
                    PriorityQueue<WorkOrderNumber, i64>,
                    Ss,
                >,
            >,
    {
        Actor::<WeeklyRequestMessage, WeeklyResponseMessage, WeeklyAlgorithm<Ss>>::builder(
        )
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

#[cfg(test)]
mod tests
{
    // use ordinator_scheduling_environment::work_order::WorkOrder;
    // use ordinator_scheduling_environment::work_order::WorkOrderNumber;
    // use ordinator_scheduling_environment::work_order::work_order_dates::unloading_point::UnloadingPoint;
    // use ordinator_scheduling_environment::worker_environment::resources::Resources;

    // TODO: Rewrite this test after determining the builder design
    // #[test]
    // fn test_extract_state_to_scheduler_overview()
    // {
    //     WorkOrder::builder(WorkOrderNumber(2100000001))
    //         .operations_builder(10, Resources::MtnMech, |e| {
    //             e.operation_info(|e| e.work_remaining(1.0))
    //                 .unloading_point(UnloadingPoint::default())
    //         })
    //         .operations_builder(20, Resources::MtnMech, |e| {
    //             e.operation_info(|e| e.work_remaining(1.0))
    //                 .unloading_point(UnloadingPoint::default())
    //         })
    //         .operations_builder(30, Resources::MtnMech, |e| {
    //             e.operation_info(|e| e.work_remaining(1.0))
    //                 .unloading_point(UnloadingPoint::default())
    //         })
    //         .build();
    // }
} // Note: All algorithm tests must be written as integration tests
