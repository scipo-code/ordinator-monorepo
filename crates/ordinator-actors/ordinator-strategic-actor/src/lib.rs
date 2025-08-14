pub mod algorithm;
//
// What
pub mod messages;

use std::fmt::Debug;
use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::Arc;
use std::sync::Mutex;

use algorithm::strategic_parameters::StrategicParameters;
use algorithm::strategic_solution::StrategicSolution;
use algorithm::StrategicAlgorithm;
use anyhow::Result;
use arc_swap::ArcSwap;
use bus::BusReader;
use flume::Sender;
use messages::StrategicRequestMessage;
use messages::StrategicResponseMessage;
use ordinator_actor_core::algorithm::Algorithm;
use ordinator_actor_core::traits::ActorBasedLargeNeighborhoodSearch;
use ordinator_actor_core::Actor;
use ordinator_configuration::SystemConfigurations;
use ordinator_orchestrator_actor_traits::ActorFactory;
use ordinator_orchestrator_actor_traits::CommandHandler;
use ordinator_orchestrator_actor_traits::Communication;
use ordinator_orchestrator_actor_traits::StateLink;
use ordinator_orchestrator_actor_traits::SystemSolutions;
use ordinator_scheduling_environment::work_order::WorkOrderNumber;
use ordinator_scheduling_environment::worker_environment::resources::ActorCompositeId;
use ordinator_scheduling_environment::SchedulingEnvironment;
use priority_queue::PriorityQueue;

pub struct StrategicActor<Ss: Debug>(
    Actor<StrategicRequestMessage, StrategicResponseMessage, StrategicAlgorithm<Ss>>,
)
where
    Ss: SystemSolutions<Strategic = StrategicSolution>,
    Actor<StrategicRequestMessage, StrategicResponseMessage, StrategicAlgorithm<Ss>>:
        CommandHandler<StrategicRequestMessage, StrategicResponseMessage>;

impl<Ss> Deref for StrategicActor<Ss>
where
    Ss: SystemSolutions<Strategic = StrategicSolution> + Debug,
    Actor<StrategicRequestMessage, StrategicResponseMessage, StrategicAlgorithm<Ss>>:
        CommandHandler<StrategicRequestMessage, StrategicResponseMessage>,
{
    type Target = Actor<StrategicRequestMessage, StrategicResponseMessage, StrategicAlgorithm<Ss>>;

    fn deref(&self) -> &Self::Target
    {
        &self.0
    }
}

impl<Ss> DerefMut for StrategicActor<Ss>
where
    Ss: SystemSolutions<Strategic = StrategicSolution> + Debug,
    Actor<StrategicRequestMessage, StrategicResponseMessage, StrategicAlgorithm<Ss>>:
        CommandHandler<StrategicRequestMessage, StrategicResponseMessage>,
{
    fn deref_mut(&mut self) -> &mut Self::Target
    {
        &mut self.0
    }
}

pub struct StrategicApi {}
impl<Ss> ActorFactory<Ss> for StrategicApi
where
    Ss: SystemSolutions<Strategic = StrategicSolution> + Send + Sync + 'static + Debug,
    Actor<StrategicRequestMessage, StrategicResponseMessage, StrategicAlgorithm<Ss>>:
        CommandHandler<StrategicRequestMessage, StrategicResponseMessage>,
{
    type Communication = Communication<StrategicRequestMessage, StrategicResponseMessage>;

    fn construct_actor(
        id: ActorCompositeId,
        scheduling_environment_guard: Arc<Mutex<SchedulingEnvironment>>,
        shared_solution_arc_swap: Arc<ArcSwap<Ss>>,
        system_configurations: Arc<ArcSwap<SystemConfigurations>>,
        state_link_bus: BusReader<StateLink>,
        error_channel: Sender<anyhow::Error>,
    ) -> Result<<Self as ActorFactory<Ss>>::Communication>
    where
        Ss: SystemSolutions<Strategic = StrategicSolution> + Send + Sync + 'static,
        StrategicAlgorithm<Ss>: ActorBasedLargeNeighborhoodSearch
            + Send
            + Sync
            + From<
                Algorithm<
                    StrategicSolution,
                    StrategicParameters,
                    PriorityQueue<WorkOrderNumber, i64>,
                    Ss,
                >,
            >,
    {
        Actor::<StrategicRequestMessage, StrategicResponseMessage, StrategicAlgorithm<Ss>>::builder(
        )
        .agent_id(id.clone())
        .scheduling_environment(Arc::clone(&scheduling_environment_guard))
        .algorithm(|ab| {
            ab.id(id)
                // So this function returns a `Result`
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

    // You should make this again at a later date.
    // TODO [ ]
    // Make this test after determining what should be done about the
    // builders
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
} // Crucial note, all algorithm tests have to be made in the integration tests
