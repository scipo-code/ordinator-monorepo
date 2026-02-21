use std::collections::HashMap;

use anyhow::Result;
use anyhow::bail;
use anyhow::ensure;
use ordinator_orchestrator_actor_traits::SystemSolutions;
use ordinator_scheduling_environment::work_order::operation::Work;
use ordinator_scheduling_environment::worker_environment::resources::Resources;
use strum::IntoEnumIterator;
use tracing::Level;
use tracing::event;

use crate::WeeklyActor;
use crate::algorithm::weekly_resources::WeeklyResources;
use crate::algorithm::weekly_solution::WeeklySolution;

#[allow(dead_code)]
pub trait WeeklyAssertions
{
    fn assert_aggregated_load(&self) -> Result<()>;
    fn assert_excluded_periods(&self) -> Result<()>;
}

impl<Ss> WeeklyAssertions for WeeklyActor<Ss>
where
    Ss: SystemSolutions<Weekly = WeeklySolution> + std::fmt::Debug,
{
    fn assert_aggregated_load(&self) -> Result<()>
    {
        let mut aggregated_weekly_load = WeeklyResources::new(HashMap::new());
        for period in &self.0.algorithm.parameters.weekly_periods {
            for (work_order_number, weekly_solution) in self
                .0
                .algorithm
                .solution
                .weekly_scheduled_work_orders
                .iter()
            {
                let weekly_parameter = self
                    .0
                    .algorithm
                    .parameters
                    .weekly_work_order_parameters
                    .get(work_order_number)
                    .unwrap();
                if weekly_solution == &period.clone() {
                    let work_load = &weekly_parameter.work_load;
                    for resource in Resources::iter() {
                        let load: Work =
                            work_load.get(&resource).cloned().unwrap_or(Work::from(0.0));
                        aggregated_weekly_load.update_load(
                            period,
                            &resource,
                            load,
                            LoadOperation::Add,
                        );
                    }
                }
            }
        }

        for (resource, periods) in aggregated_weekly_load.inner {
            for (period, load) in periods.0 {
                match self
                    .weekly_algorithm
                    .resources_loadings()
                    .inner
                    .get(&resource)
                    .unwrap()
                    .0
                    .get(&period)
                {
                    // Some(resource_load) if (*resource_load - load).abs() < 0.005 => continue,
                    Some(resource_load) => {
                        if resource_load.0.round_dp(6) != load.0.round_dp(6) {
                            event!(Level::ERROR, resource = %resource, period = %period, aggregated_load = %load, resource_load = %resource_load);
                            bail!("aggregated load and loading are not the same");
                        }
                    }
                    None => {
                        bail!("aggregated load and resource loading are not identically shaped")
                    }
                }
            }
        }
        Ok(())
    }

    fn assert_excluded_periods(&self) -> Result<()>
    {
        for (work_order_number, weekly_parameter) in
            &self.0.algorithm.parameters.weekly_work_order_parameters
        {
            let excluded_periods = &weekly_parameter.excluded_periods;
            let locked_in_period = &weekly_parameter.locked_in_period;

            let scheduled_period = self
                .0
                .algorithm
                .solution
                .weekly_scheduled_work_orders
                .get(work_order_number)
                .unwrap();

            if let Some(period) = scheduled_period.weekly_forced() {
                ensure!(
                    !excluded_periods.contains(period),
                    "\n{:#?}\nscheduled in:{:#?}\nlocked_in_period\n{:#?}\nwhich is part of the excluded periods:\n{:#?}",
                    work_order_number,
                    period,
                    locked_in_period,
                    excluded_periods,
                );
            }
            if let Some(locked_in_period) = locked_in_period {
                ensure!(!excluded_periods.contains(locked_in_period))
            }
        }
        Ok(())
    }
}

// impl TestAlgorithm for WeeklyAgent {
//     type InfeasibleCases = WeeklyInfeasibleCases;

//     fn determine_algorithm_state(&self) ->
// AlgorithmState<Self::InfeasibleCases> {         let scheduling_environment =
// self.scheduling_environment.lock().unwrap();

//         let mut weekly_state =
// AlgorithmState::Infeasible(Self::InfeasibleCases::default());

//         for (work_order_number, scheduled_period) in self
//             .weekly_agent_algorithm
//             .weekly_project_solution_arc_swap
//             .0
//             .load()
//             .weekly
//             .scheduled_periods
//         {
//             let scheduled_period = scheduled_period.clone();
//             let work_order = scheduling_environment
//                 .work_orders()
//                 .inner
//                 .get(&work_order_number)
//                 .unwrap();

//             let first_period =
// self.weekly_agent_algorithm.periods().first().unwrap();

//             let basic_start_of_first_activity =
// work_order.order_dates().basic_start_date;

//             let awsc = work_order.work_order_analytic.user_status_codes.awsc;

//             match scheduled_period {
//                 Some(scheduled_period) => {
//                     if awsc
//                         &&
// !(scheduled_period.contains_date(basic_start_of_first_activity)
// || work_order.unloading_point_contains_period(scheduled_period.clone()))
//                         && &basic_start_of_first_activity >
// &first_period.start_date().date_naive()                     {
//
// weekly_state.infeasible_cases_mut().unwrap().respect_awsc =
// ConstraintState::Infeasible(format!(                                 "Work
// order {:?} does not respect AWSC. Period: {}, basic start date: {}, status
// codes: {:?}, unloading_point: {:?}, vendor: {}",
// work_order_number,                                 scheduled_period,
//                                 basic_start_of_first_activity,
//
// work_order.work_order_analytic.user_status_codes,
// work_order.operations.values().map(|opr| opr.unloading_point.period.clone()),
//                                 if work_order.is_vendor() { "VEN" } else { "
// " },                             ));
//                         break;
//                     }
//                 }
//                 None => {
//
// weekly_state.infeasible_cases_mut().unwrap().respect_awsc =
// ConstraintState::Infeasible(format!(                             "Work order
// {:?} does not have a period",                             work_order_number,
//                         ));
//                     break;
//                 }
//             }
//             weekly_state.infeasible_cases_mut().unwrap().respect_awsc =
//                 ConstraintState::Feasible;
//         }

//         for (work_order_number, optimized_work_order) in
//             self.weekly_agent_algorithm.optimized_work_orders()
//         {
//             let work_order = scheduling_environment
//                 .work_orders()
//                 .inner
//                 .get(work_order_number)
//                 .unwrap();

//             let periods = scheduling_environment.periods();

//             if work_order.unloading_point().is_some()
//                 && work_order.unloading_point() !=
// optimized_work_order.scheduled_period                 &&
// !periods[0..=1].contains(work_order.unloading_point().as_ref().unwrap())
//                 && !work_order.work_order_analytic.user_status_codes.awsc
//                 && !work_order.work_order_analytic.user_status_codes.sch
//             {
//                 error!(
//                     work_order_number = ?work_order_number,
//                     work_order_unloading_point =
// ?work_order.unloading_point(),                     work_order_status_codes =
// ?work_order.work_order_analytic.user_status_codes,
// work_order_dates = ?work_order.order_dates().basic_start_date,
// periods = ?periods[0..=1],
// optimized_work_order_scheduled_period =
// ?optimized_work_order.scheduled_period,
// optimized_work_order_locked_in_period =
// ?optimized_work_order.locked_in_period,                 );
//                 weekly_state
//                     .infeasible_cases_mut()
//                     .unwrap()
//                     .respect_unloading = ConstraintState::Infeasible(format!(
//                     "\t\t\nWork order number: {:?}\t\t\nwith unloading
// period: {}\t\t\nwith scheduled period: {}\t\t\nwith locked period: {}",
//                     work_order_number,
//                     work_order.unloading_point().as_ref().unwrap(),
//                     optimized_work_order.scheduled_period.clone().unwrap(),
//                     optimized_work_order.locked_in_period.clone().unwrap(),
//                 ));
//                 break;
//             }
//             weekly_state
//                 .infeasible_cases_mut()
//                 .unwrap()
//                 .respect_unloading = ConstraintState::Feasible;
//         }

//         for (work_order_number, scheduled_period) in self
//             .weekly_agent_algorithm
//             .weekly_project_solution_arc_swap
//             .0
//             .load()
//             .weekly
//             .scheduled_periods
//         {
//             let work_order = scheduling_environment
//                 .work_orders()
//                 .inner
//                 .get(&work_order_number)
//                 .unwrap();

//             let periods = scheduling_environment.periods();

//             if work_order.work_order_analytic.user_status_codes.sch
//                 && work_order.unloading_point().is_some()
//                 &&
// periods[0..=1].contains(work_order.unloading_point().as_ref().unwrap())
//                 && scheduled_period != work_order.unloading_point()
//             {
//                 error!(
//                     work_order_number = ?work_order_number,
//                     work_order_unloading_point =
// ?work_order.unloading_point(),                     work_order_status_codes =
// ?work_order.work_order_analytic.user_status_codes,
// work_order_dates = ?work_order.order_dates().basic_start_date,
// periods = ?periods[0..=1],
// optimized_work_order_scheduled_period = ?scheduled_period,
// optimized_work_order_locked_in_period = ?self.locked_in_period,
// );                 weekly_state
//                     .infeasible_cases_mut()
//                     .unwrap()
//                     .respect_sch = ConstraintState::Infeasible(format!(
//                     "\t\t\nWork order number: {:?}\t\t\nwith scheduled
// period: {}\t\t\nwith locked period: {:?}\t\t\n work order status codes:
// {:?}\t\t\n work order unloading point: {:?}",
// work_order_number,
// scheduled_period.scheduled_period.as_ref().unwrap(),
// scheduled_period.locked_in_period.as_ref(),
// work_order.work_order_analytic.user_status_codes,
// work_order.unloading_point().as_ref(),                 ));
//                 break;
//             }
//             weekly_state.infeasible_cases_mut().unwrap().respect_sch =
// ConstraintState::Feasible;         }

//         weekly_state
//     }
// }
