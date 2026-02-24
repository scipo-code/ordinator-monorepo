use std::collections::HashMap;
use std::collections::hash_map::Entry;

use anyhow::Result;
use anyhow::bail;
use anyhow::ensure;
use ordinator_actor_core::algorithm::Algorithm;
use ordinator_orchestrator_actor_traits::SystemSolutions;
use ordinator_orchestrator_actor_traits::WhereIsWorkOrder;
use ordinator_scheduling_environment::work_order::WorkOrderNumber;
use ordinator_scheduling_environment::work_order::operation::Work;
use ordinator_scheduling_environment::worker_environment::resources::Skill;
use priority_queue::PriorityQueue;
use strum::IntoEnumIterator;
use tracing::Level;
use tracing::event;

use super::weekly_parameters::WeeklyParameters;
use super::weekly_resources::WeeklyResources;
use super::weekly_solution::WeeklySolution;

#[allow(dead_code)]
pub trait WeeklyAssertions
{
    fn assert_that_capacity_is_respected(
        weekly_loading: &WeeklyResources,
        weekly_capacity: &WeeklyResources,
    ) -> Result<()>;
    fn assert_aggregated_load(&self) -> Result<()>;
    fn assert_excluded_periods(&self) -> Result<()>;
}

impl<Ss> WeeklyAssertions
    for Algorithm<WeeklySolution, WeeklyParameters, PriorityQueue<WorkOrderNumber, u64>, Ss>
where
    Ss: SystemSolutions,
{
    fn assert_that_capacity_is_respected(
        weekly_loading: &WeeklyResources,
        weekly_capacity: &WeeklyResources,
    ) -> Result<()>
    {
        for (period, skill_loadings) in weekly_loading.0.iter() {
            for (skill, loading) in skill_loadings.iter() {
                let capacity = weekly_capacity
                    .0
                    .get(period)
                    .unwrap()
                    .get(skill)
                    .copied()
                    .unwrap_or(Work::from(0.0));
                if *loading > capacity {
                    event!(
                        Level::ERROR,
                        skill = ?skill,
                        period = ?period,
                        capacity = ?capacity,
                        loading = ?loading,
                        "weekly_resources_exceeded"
                    );
                    bail!("Capacity exceeded")
                }
            }
        }
        Ok(())
    }

    fn assert_aggregated_load(&self) -> Result<()>
    {
        let mut aggregated_weekly_load = HashMap::new();
        for period in &self.parameters.weekly_periods {
            for (work_order_number, weekly_solution) in self.solution.every_work_order().iter() {
                let weekly_parameter = self
                    .parameters
                    .weekly_work_order_parameters
                    .get(work_order_number)
                    .unwrap();

                // Weekly load aggregation is used to test internal WeeklySolution consistency.
                // Do not include ProjectSolutions here.
                if let WhereIsWorkOrder::Weekly(weekly_scheduled_period) = weekly_solution
                    && weekly_scheduled_period == period
                {
                    let work_load = &weekly_parameter.work_load;
                    for resource in Skill::iter() {
                        let load: Work =
                            work_load.get(&resource).cloned().unwrap_or(Work::from(0.0));
                        // Verify total hours; individual resource validation handled separately.

                        match aggregated_weekly_load.entry((period, resource)) {
                            Entry::Occupied(mut occupied_entry) => {
                                *occupied_entry.get_mut() += load;
                            }
                            Entry::Vacant(vacant_entry) => {
                                vacant_entry.insert(load);
                            }
                        }
                    }
                }
            }
        }

        // Verify aggregated total_hours match actual loadings.
        for (resource, total_work) in aggregated_weekly_load {
            let loadings = self
                .solution
                .weekly_loadings
                .0
                .get(resource.0)
                .unwrap()
                .get(&resource.1)
                .copied()
                .unwrap_or(Work::from(0.0));

            ensure!(loadings == total_work);
        }
        Ok(())
    }

    fn assert_excluded_periods(&self) -> Result<()>
    {
        for (work_order_number, weekly_parameter) in
            &self.parameters.weekly_work_order_parameters
        {
            let excluded_periods = &weekly_parameter.excluded_periods;
            let locked_in_period = &weekly_parameter.locked_in_period;

            let scheduled_period = self
                .solution
                .every_work_order()
                .get(work_order_number)
                .unwrap();

            // Use [`WhereIsWorkOrder`] to validate weekly period constraints.
            if let WhereIsWorkOrder::Weekly(period) = scheduled_period {
                ensure!(
                    !excluded_periods.contains(period),
                    "\n{:#?}\nscheduled in:{:#?}\nlocked_in_period\n{:#?}\nwhich is part of the excluded periods:\n{:#?}",
                    work_order_number,
                    period,
                    locked_in_period,
                    excluded_periods,
                );
            }
            if let WhereIsWorkOrder::Weekly(locked_in_period) = locked_in_period {
                ensure!(!excluded_periods.contains(locked_in_period))
            }
        }
        Ok(())
    }
}
