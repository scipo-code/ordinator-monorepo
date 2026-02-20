use std::collections::HashSet;
use std::fmt::Debug;

use anyhow::bail;
use anyhow::Result;
use ordinator_actor_core::Actor;
use ordinator_orchestrator_actor_traits::WeeklyInterface;
use ordinator_orchestrator_actor_traits::SystemSolutions;
use ordinator_scheduling_environment::work_order::WorkOrderNumber;
use tracing::event;
use tracing::Level;

use crate::algorithm::supervisor_solution::DailySolution;
use crate::algorithm::DailyAlgorithm;
use crate::messages::DailyRequestMessage;
use crate::messages::DailyResponseMessage;

#[allow(dead_code)]
pub trait DailyAssertions
{
    fn test_symmetric_difference_between_project_operations_and_operational_state_machine(
        &self,
    ) -> Result<()>;
    fn assert_operational_state_machine_woas_is_subset_of_project_shared_solution(
        &self,
    ) -> Result<()>;
}

impl<Ss> DailyAssertions
    for Actor<DailyRequestMessage, DailyResponseMessage, DailyAlgorithm<Ss>>
where
    Ss: SystemSolutions<Daily = DailySolution> + Debug,
    DailyResponseMessage: Debug,
{
    fn test_symmetric_difference_between_project_operations_and_operational_state_machine(
        &self,
    ) -> Result<()>
    {
        let project_operation_woas: HashSet<WorkOrderNumber> = self
            .algorithm
            .loaded_system_solution
            .strategic()?
            .supervisor_tasks(&self.algorithm.parameters.supervisor_periods)
            .iter()
            .map(|f| *f.0)
            .collect();

        let operational_state_woas: HashSet<WorkOrderNumber> = self
            .algorithm
            .solution
            .get_iter()
            .map(|(woa, _)| woa.1 .0)
            .collect();

        let symmetric_difference = project_operation_woas
            .symmetric_difference(&operational_state_woas)
            .cloned()
            .collect::<HashSet<WorkOrderNumber>>();

        if !symmetric_difference.is_empty() {
            event!(Level::ERROR,
                non_corresponding_work_order_activities = ? symmetric_difference,
                in_the_project_operations = ?symmetric_difference.intersection(&project_operation_woas),
                in_the_operational_state_woas = ?symmetric_difference.intersection(&operational_state_woas),
            );
            bail!(
                "If the symmetric difference is empty it means that there are state inconsistencies"
            );
        }
        Ok(())
    }

    // Assert that operational state machine work orders are a subset of project operations
    fn assert_operational_state_machine_woas_is_subset_of_project_shared_solution(
        &self,
    ) -> Result<()>
    {
        let strategic_work_orders: HashSet<WorkOrderNumber> = self
            .algorithm
            .loaded_system_solution
            .strategic()?
            .supervisor_tasks(&self.algorithm.parameters.supervisor_periods)
            .iter()
            .map(|f| *f.0)
            .collect();

        let operational_state_work_order_activities: HashSet<WorkOrderNumber> = self
            .algorithm
            .solution
            .get_iter()
            .map(|(woa, _)| woa.1 .0)
            .collect();

        if !operational_state_work_order_activities.is_subset(&strategic_work_orders) {
            event!(
                Level::ERROR,
                operational_difference_with_project_operations = ?operational_state_work_order_activities
                    .difference(&strategic_work_orders)
                    .cloned()
                    .collect::<HashSet<_>>()
            );
            bail!(
                "The project_operations should always hold all the work_order_activities of the operational_state_machine"
            );
        }
        Ok(())
    }
}
