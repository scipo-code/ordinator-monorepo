use std::borrow::Cow;
use std::collections::HashMap;

use anyhow::Context;
use anyhow::Result;
use anyhow::bail;
use anyhow::ensure;
use colored::Colorize;
use ordinator_orchestrator_actor_traits::SystemSolutions;
use ordinator_orchestrator_actor_traits::WhereIsWorkOrder;
use ordinator_scheduling_environment::time_environment::day::Day;
use ordinator_scheduling_environment::time_environment::day::Days;
use ordinator_scheduling_environment::work_order::WorkOrderNumber;
use ordinator_scheduling_environment::work_order::operation::Work;
use ordinator_scheduling_environment::worker_environment::resources::Resources;
use priority_queue::PriorityQueue;
use strum::IntoEnumIterator;
use tracing::Level;
use tracing::event;

use super::Algorithm;
use super::tactical_parameters::TacticalParameters;
use super::tactical_solution::TacticalSolution;
use super::tactical_solution::TacticalWhereIsWorkOrder;

type TotalExcessHours = Work;

#[allow(dead_code)]
pub trait TacticalAssertions
{
    fn assert_that_loading_matches_scheduled(&self) -> Result<()>;

    fn asset_that_capacity_is_not_exceeded(&self) -> Result<TotalExcessHours>;

    fn assert_that_total_loading_is_equal_to_total_scheduled(&self) -> Result<TotalExcessHours>;
}

impl<Ss> TacticalAssertions
    for Algorithm<TacticalSolution, TacticalParameters, PriorityQueue<WorkOrderNumber, u64>, Ss>
where
    Ss: SystemSolutions,
{
    fn assert_that_loading_matches_scheduled(&self) -> Result<()>
    {
        let mut aggregated_load: HashMap<Resources, HashMap<Day, Work>> = HashMap::new();

        for (_work_order_number, solution) in &self
            .solution
            .tactical_work_orders
            .0
            .iter()
            .filter(|(_, whe_tac_sch)| whe_tac_sch.is_tactical())
            .collect::<Vec<_>>()
        {
            for operation_solution in solution.tactical_operations()?.0.values() {
                let resource = &operation_solution.resource;

                for (day, load) in &operation_solution.scheduled {
                    *aggregated_load
                        .entry(*resource)
                        .or_default()
                        .entry(day.clone())
                        .or_insert(Work::from(0.0)) += load;
                }
            }
        }

        for resource in Resources::iter() {
            for day in self.parameters.tactical_days.iter().enumerate() {
                let resource_map = match aggregated_load.get(&resource) {
                    Some(map) => Cow::Borrowed(map),
                    None => Cow::Owned(HashMap::new()),
                };

                let zero_work = Work::from(0.0);
                let agg_load = resource_map.get(day.1).unwrap_or(&zero_work);
                let sch_load = self
                    .solution
                    .tactical_loadings
                    .get_resource(&resource, day.0)
                    // The important thing here is that the asserts should not know
                    // about the initialization of the
                    .unwrap_or(&zero_work);

                // info!(target: "debug", day = ?day.0, ?agg_load, ?sch_load);
                event!(target: "debug", Level::ERROR, agg_load = ?agg_load, sch_load = ?sch_load, resource = ?resource, day = ?day);
                if (agg_load - sch_load).0.round_dp(9) != Work::from(0.0).0 {
                    event!(target: "debug", Level::ERROR, agg_load = ?agg_load, sch_load = ?sch_load, resource = ?resource, day = ?day);
                    bail!(
                        "Loads does not match on: \n\tday {}\n\tresource: {}\n\tscheduled load (based on loadings): {}\n\taggregated_load (based on scheduled work): {}\n",
                        day.1.to_string().bright_green(),
                        resource.to_string().bright_blue(),
                        sch_load.to_string().bright_yellow(),
                        agg_load.to_string().bright_yellow()
                    );
                }
            }
        }

        Ok(())
    }

    fn asset_that_capacity_is_not_exceeded(&self) -> Result<TotalExcessHours>
    {
        let mut total_excess_hours = Work::from(0.0);
        for (resource, days) in &self.solution.tactical_loadings.resources {
            let capacity_days = self
                .parameters
                .tactical_capacity
                .resources
                .get(resource)
                .cloned()
                .unwrap_or(Days::zero_from_existing(days));

            for (load, capacity) in days.days.iter().zip(capacity_days.days) {
                // let capacity = self
                //     .parameters
                //     .tactical_capacity
                //     .resources.
                //     .get_resource(resource, day)?;

                total_excess_hours += (*load - capacity).max(Work::from(0.0));
                // ensure!(
                //     load <= capacity,
                //     format!(
                //         "Load exceeds Capacity for resource: {:?} on day:
                // {:?} with load {:?} and capacity {:?}",
                //         resource, day, load, capacity
                //     )
                // );
            }
        }
        Ok(total_excess_hours)
    }

    fn assert_that_total_loading_is_equal_to_total_scheduled(&self) -> Result<TotalExcessHours>
    {
        let loadings_work = self.solution.tactical_loadings.total_hours();

        let mut scheduled_work = Work::from(0.0);
        // Work::from(0.0),
        for work_order_solution in &self.solution.tactical_work_orders.0 {
            let work_order_work = &self
                .parameters
                .tactical_work_orders
                .get(work_order_solution.0)
                .expect("Parameters should always cover the Solution")
                .tactical_operation_parameters;

            if let WhereIsWorkOrder::Tactical(wo) = work_order_solution.1 {
                // TODO [ ] Add the parameters here instead.
                // ESSAY:
                // I think that what is happening is the tactical stops when it cannot find room
                // for an Operation, where it should instead continue.
                for (activity_number, operation_parameter) in work_order_work.iter() {
                    let work_remaining = operation_parameter.work_remaining;

                    let operation_solution_work = wo.0.get(activity_number).context("Every activity of a work order should be present in a Tactical solution work order")?
                        .scheduled
                        .iter()
                        .fold(Work::from(0.0), |acc, e| acc + e.1);

                    ensure!(
                        work_remaining == operation_solution_work,
                        "work scheduled for: {:?} - {:?} for {:?}\nhours: {operation_solution_work:>4} in operation solution should always match\nhours: {work_remaining:>4} in operation parameter",
                        work_order_solution.0,
                        activity_number,
                        operation_parameter.resource
                    );
                    scheduled_work += operation_solution_work;
                }
            }
        }

        ensure!(
            loadings_work.round() == scheduled_work.round(),
            format!(
                "Tactical Loadings does not match Tactical scheduled work\nScheduled work: {scheduled_work}\nLoadings work: {loadings_work}"
            )
        );
        Ok(scheduled_work)
    }
}

//         algorithm_state
//             .infeasible_cases_mut()
//             .unwrap()
//             .earliest_start_day = (|| {
//             for (work_order_number, optimized_work_order) in
// self.parameters().clone() {                 let start_date_from_period =
// match optimized_work_order.scheduled_period {
// Some(period) => period.start_date().date_naive(),                     None =>
// optimized_work_order.earliest_allowed_start_date,                 };

//                 if let Some(operation_solutions) =
// optimized_work_order.operation_solutions {                     let
// start_days: Vec<_> =                         operation_solutions
//                             .values()
//                             .map(|operation_solution| {
//                                 operation_solution
//                             .scheduled
//                             .first()
//                             .expect("All scheduled operations should have a
// first scheduled day")                             .0
//                             .clone()
//                             })
//                             .collect();

//                     for start_day in start_days {
//                         if start_day.date().date_naive() <
// start_date_from_period {                             error!(start_day =
// ?start_day.date(), start_date_from_period = ?start_date_from_period);
//                             return ConstraintState::Infeasible(format!(
//                                 "{:?} is outside of its earliest start day:
// {}",                                 work_order_number,
// start_date_from_period                             ));
//                         }
//                     }
//                 }
//             }
//             ConstraintState::Feasible
//         })();

//         algorithm_state
//             .infeasible_cases_mut()
//             .unwrap()
//             .all_scheduled = (|| {
//             for (work_order_number, optimized_work_order) in
// self.parameters().clone() {                 if
// optimized_work_order.operation_solutions.is_none() {
// return ConstraintState::Infeasible(work_order_number.0.to_string());
//                 }
//             }
//             ConstraintState::Feasible
//         })();

//         algorithm_state
//             .infeasible_cases_mut()
//             .unwrap()
//             .respect_period_id = (|| {
//             for (_work_order_number, optimized_work_order) in
// self.parameters().clone() {                 let scheduled_period = match
// optimized_work_order.scheduled_period {                     Some(period) =>
// period,                     None => return ConstraintState::Feasible,
//                 };
//                 if !self.tactical_periods.contains(&scheduled_period) {
//                     error!(work_order_number = ?_work_order_number,
// scheduled_period = ?scheduled_period, tactical_periods =
// ?self.tactical_periods, "Tactical period does not contain the scheduled
// period of the tactical work order");                     return
// ConstraintState::Infeasible(format!(                         "{:?} has a
// wrong scheduled period {}",                         _work_order_number,
// scheduled_period                     ));
//                 }
//             }
//             ConstraintState::Feasible
//         })();

//         let infeasible_cases =
// algorithm_state.infeasible_cases_mut().unwrap();

//         if infeasible_cases.aggregated_load == ConstraintState::Feasible
//             && infeasible_cases.earliest_start_day ==
// ConstraintState::Feasible             && infeasible_cases.all_scheduled ==
// ConstraintState::Feasible             && infeasible_cases.respect_period_id
// == ConstraintState::Feasible         {
//             AlgorithmState::Feasible
//         } else {
//             algorithm_state
//         }
//     }
// }
