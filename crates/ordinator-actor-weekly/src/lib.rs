pub mod algorithm;
// pub mod assert_functions;
pub mod messages;

use std::fmt::Debug;
use std::ops::Deref;
use std::ops::DerefMut;

use algorithm::WeeklyAlgorithm;
use algorithm::weekly_solution::WeeklySolution;
use messages::WeeklyRequestMessage;
use messages::WeeklyResponseMessage;
use ordinator_actor_core::Actor;
use ordinator_orchestrator_actor_traits::CommandHandler;
use ordinator_orchestrator_actor_traits::SystemSolutions;

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
