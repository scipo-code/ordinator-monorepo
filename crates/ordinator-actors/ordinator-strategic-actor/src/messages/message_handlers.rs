use std::any::type_name;
use std::fmt::Debug;

use anyhow::Context;
use anyhow::Result;
use anyhow::bail;
use colored::Colorize;
use ordinator_actor_core::Actor;
use ordinator_actor_core::traits::ActorBasedLargeNeighborhoodSearch;
use ordinator_orchestrator_actor_traits::CommandHandler;
use ordinator_orchestrator_actor_traits::StateLink;
use ordinator_orchestrator_actor_traits::SystemSolutions;
use tracing::Level;
use tracing::event;

use super::StrategicRequestMessage;
use super::StrategicResponseMessage;
use super::StrategicSchedulingEnvironmentCommands;
use super::StrategicStatusMessage;
use crate::algorithm::StrategicAlgorithm;
use crate::algorithm::strategic_parameters::WorkOrderParameter;
use crate::algorithm::strategic_resources::StrategicResources;
use crate::algorithm::strategic_solution::StrategicSolution;
use crate::messages::StrategicRequestScheduling;
use crate::messages::StrategicResponseScheduling;

// TODO: Refactor type system to resolve blanket implementation conflicts with Orphan Rule.
// The LNS blanket implementation on inner types conflicts with actor-level implementations.
impl<Ss> CommandHandler<StrategicRequestMessage, StrategicResponseMessage>
    for Actor<StrategicRequestMessage, StrategicResponseMessage, StrategicAlgorithm<Ss>>
where
    Ss: SystemSolutions<Strategic = StrategicSolution> + Debug,
{
    fn handle_request_message(
        &mut self,
        strategic_request_message: StrategicRequestMessage,
    ) -> Result<StrategicResponseMessage>
    {
        let strategic_response = match strategic_request_message {
            ordinator_actor_core::RequestMessage::Status(strategic_status_message) => {
                match strategic_status_message {
                    StrategicStatusMessage::General => {
                        // TODO: Implement general status message handling
                        Err(anyhow::anyhow!(
                            "Implement this. And please, do it correctly and thoughtfully the first time"
                        ))
                    }
                    // Consider moving period creation to the Orchestrator
                    StrategicStatusMessage::Period(period) => {
                        if !self
                            .algorithm
                            .parameters
                            .strategic_periods
                            .iter()
                            .map(|period| period.period_string())
                            .collect::<Vec<_>>()
                            .contains(&period)
                        {
                            bail!("Period not found in the the scheduling environment".to_string());
                        }

                        // let work_orders_by_period: HashMap<WorkOrderNumber, WorkOrderResponse> =
                        //     self.algorithm
                        //         .solution
                        //         .strategic_scheduled_work_orders
                        //         .iter()
                        //         .filter(|(_, sch_per)| match sch_per {
                        //             Some(scheduled_period) => {
                        //                 scheduled_period.period_string() == period
                        //             }
                        //             None => false,
                        //         })
                        //         .map(|(work_order_number, _)| {
                        //             let work_orders =
                        //                 self.scheduling_environment.lock().unwrap().work_orders;

                        //             let work_order_configurations =
                        //                 &work_orders.work_order_configurations;

                        //             let work_order =
                        //
                        // &work_orders.inner.get(work_order_number).unwrap().clone();

                        //             let work_order_response = WorkOrderResponse::new(
                        //                 work_order,
                        //                 (**self.algorithm.loaded_shared_solution).clone().into(),
                        //                 &self
                        //                     .scheduling_environment
                        //                     .lock()
                        //                     .unwrap()
                        //                     .time_environment
                        //                     .strategic_periods
                        //                     .clone(),
                        //                 work_order_configurations,
                        //             );
                        //             (*work_order_number, work_order_response)
                        //         })
                        //         .collect();

                        bail!("The endpoints are being refactored")
                    }
                    // Use `From` implementations for work order conversion
                    StrategicStatusMessage::WorkOrder(_work_order_number) => {
                        // TODO [ ]
                        // Make a `From` implementation.
                        // let strategic_solution_for_specific_work_order = self
                        //     .algorithm
                        //     .solution
                        //     .strategic_scheduled_work_orders
                        //     .get(&work_order_number)
                        //     .with_context(|| format!("{:?} not found in", work_order_number,))?;

                        // let strategic_parameter = self
                        //     .algorithm
                        //     .parameters
                        //     .strategic_work_order_parameters
                        //     .get(&work_order_number)
                        //     .with_context(|| {
                        //         format!(
                        //             "{:?} does not have a {}",
                        //             work_order_number,
                        //             std::any::type_name::<WorkOrderParameter>(),
                        //         )
                        //     })?;

                        // let locked_in_period = &strategic_parameter.locked_in_period;
                        // let excluded_from_period = &strategic_parameter.excluded_periods;

                        // let strategic_api_solution = StrategicApiSolution {
                        //     solution: strategic_solution_for_specific_work_order.clone(),
                        //     locked_in_period: locked_in_period.clone(),
                        //     excluded_from_period: excluded_from_period.clone(),
                        // };

                        // let work_orders_in_period =
                        //     WorkOrdersStatus::SingleSolution(strategic_api_solution);

                        // let strategic_response_message =
                        //     StrategicResponseMessage::WorkOrder(work_orders_in_period);

                        // Ok(strategic_response_message)
                        todo!()
                    }
                }
            }
            ordinator_actor_core::RequestMessage::Scheduling(scheduling_message) => {
                let scheduling_output: StrategicResponseScheduling = self
                    .algorithm
                    .update_scheduling_state(scheduling_message)
                    .with_context(|| {
                        format!(
                            "{} was not Resolved",
                            type_name::<StrategicRequestScheduling>()
                                .split("::")
                                .last()
                                .unwrap()
                                .bright_red()
                        )
                    })?;

                self.algorithm.calculate_objective_value()?;
                event!(target: "research", Level::INFO, strategic_objective_value = ?self.algorithm.solution.objective_value());
                Ok(StrategicResponseMessage::Scheduling(scheduling_output))
            }
            ordinator_actor_core::RequestMessage::Resource(resources_message) => {
                let resources_output = self.algorithm.update_resources_state(resources_message);

                self.algorithm.calculate_objective_value()?;
                event!(target: "research", Level::INFO, strategic_objective_value = ?self.algorithm.solution.objective_value());
                Ok(StrategicResponseMessage::Resources(
                    resources_output.unwrap(),
                ))
            }
            ordinator_actor_core::RequestMessage::Time(_periods_message) => {
                // let mut scheduling_environment_guard =
                // self.scheduling_environment.lock().unwrap();

                // let periods = &mut scheduling_environment_guard
                //     .time_environment
                //     .strategic_periods;

                // for period_id in periods_message.periods.iter() {
                //     if periods.last().unwrap().id() + 1 == *period_id {
                //         let new_period =
                //             periods.last().unwrap().clone() + chrono::Duration::weeks(2);
                //         periods.push(new_period);
                //     } else {
                //         event!(Level::ERROR, "periods not handled correctly");
                //     }
                // }
                // // It should not happen like this. I think that the periods should be
                // // created through the
                // self.algorithm.parameters.strategic_periods = periods.to_vec();
                // let strategic_response_periods =
                // StrategicResponsePeriods::new(periods.clone());
                // Ok(StrategicResponseMessage::Periods(
                //     strategic_response_periods,
                // ))
                todo!()
            }
            ordinator_actor_core::RequestMessage::SchedulingEnvironment(
                strategic_scheduling_environment_commands,
            ) => match strategic_scheduling_environment_commands {
                // TODO: Move user status handling to Orchestrator. This handler should only read
                // SchedulingEnvironment and update StrategicParameters, delegating business logic
                // elsewhere through DTO-based approaches.
                StrategicSchedulingEnvironmentCommands::UserStatus(
                    _strategic_user_status_codes,
                ) => {
                    // let scheduling_environment_lock =
                    //     &mut self.scheduling_environment.lock().unwrap();

                    // for work_order_number in &strategic_user_status_codes.work_order_numbers {
                    //     let work_order = scheduling_environment_lock
                    //         .work_orders
                    //         .inner
                    //         .get_mut(work_order_number)
                    //         .with_context(|| {
                    //             format!(
                    //                 "{:?} is not found for {:?}",
                    //                 work_order_number,
                    //                 self.actor_id.asset()
                    //             )
                    //         })?;

                    //     // This should ideally be encapsulated into the a method on the WorkOrder
                    //     // that accepts a StrategicUserStatusCodes
                    //     let user_status_codes =
                    //         &mut work_order.work_order_analytic.user_status_codes;

                    //     if let Some(sece) = strategic_user_status_codes.sece {
                    //         user_status_codes.sece = sece;
                    //     }
                    //     if let Some(sch) = strategic_user_status_codes.sch {
                    //         user_status_codes.sch = sch;
                    //     }
                    //     if let Some(awsc) = strategic_user_status_codes.awsc {
                    //         user_status_codes.awsc = awsc;
                    //     }

                    // TODO: Encapsulate Algorithm to prevent direct mutation. Message handlers should not
                    // access internal fields. Instead, implement InteractWithSchedulingEnvironment trait
                    // for each Actor type to provide controlled access to state modifications.
                    //     let last_period =
                    //         self.algorithm.parameters.strategic_periods.
                    // last().cloned();

                    //     let unscheduled_period = self
                    //         .algorithm
                    //         .solution
                    //         .strategic_scheduled_work_orders
                    //         .insert(*work_order_number,
                    // last_period.clone())
                    //         .expect("WorkOrderNumber should always be
                    // present")         .expect(
                    //             "All WorkOrders should be scheduled in
                    // between ScheduleIteration loops",
                    //         );

                    //     let work_load = self
                    //         .algorithm
                    //         .parameters
                    //         .strategic_work_order_parameters
                    //         .get(work_order_number)
                    //         .unwrap()
                    //         .work_load
                    //         .clone();

                    //     let unscheduled_resources = self
                    //         .algorithm
                    //         .determine_best_permutation(
                    //             work_load.clone(),
                    //             &unscheduled_period,
                    //             ScheduleWorkOrder::Unschedule,
                    //         )
                    //         .with_context(|| {
                    //             format!(
                    //                 "{:?}\nin period {:?}\ncould not be
                    // {:?}",
                    // work_order_number,
                    //                 unscheduled_period,
                    //                 ScheduleWorkOrder::Unschedule
                    //             )
                    //         })?
                    //         .expect("It should always be possible to
                    // release resources");

                    //     self.algorithm
                    //         .update_loadings(unscheduled_resources,
                    // LoadOperation::Sub);

                    //     let scheduled_resources = self
                    //         .algorithm
                    //         .determine_best_permutation(
                    //             work_load,
                    //             &last_period.unwrap(),
                    //             ScheduleWorkOrder::Forced,
                    //         )
                    //         .with_context(|| {
                    //             format!(
                    //                 "{:?}\nin period {:?}\ncould not be
                    // {:?}",
                    // work_order_number,
                    //                 unscheduled_period,
                    //                 ScheduleWorkOrder::Forced
                    //             )
                    //         })?
                    //         .expect("It should always be possible to
                    // release resources");

                    //     self.algorithm
                    //         .update_loadings(scheduled_resources,
                    // LoadOperation::Add);
                    // FIX
                    // }

                    // TODO: Remove work order update notification - Actors no longer handle this responsibility
                    // let asset = self.actor_id.asset().clone();

                    // self.notify_orchestrator
                    //     .notify_all_agents_of_work_order_change(
                    //         strategic_user_status_codes.work_order_numbers,
                    //         &asset,
                    //     )
                    //     .context("Could not notify Orchestrator")?;

                    Ok(StrategicResponseMessage::Success)
                }
            },
            ordinator_actor_core::RequestMessage::Update => todo!(),
        };
        self.algorithm.calculate_objective_value()?;

        strategic_response
    }

    fn handle_state_link(&mut self, msg: StateLink) -> Result<StrategicResponseMessage>
    {
        match msg {
            StateLink::WorkOrders(changed_work_orders) => {
                for work_order_number in changed_work_orders {
                    let scheduling_environment_guard = self.scheduling_environment.lock().unwrap();
                    let work_order =
                        scheduling_environment_guard
                            .work_orders
                            .inner
                            .get(&work_order_number)
                            .with_context(|| {
                                format!(
                                    "{work_order_number:?} is not present in SchedulingEnvironment",
                                )
                            })?;
                    let work_order_configurations =
                        &scheduling_environment_guard.work_order_policies;
                    let material_to_period = &scheduling_environment_guard
                        .material_repo
                        .material_to_period;

                    let strategic_parameter = WorkOrderParameter::builder()
                        .with_scheduling_environment(
                            work_order,
                            &scheduling_environment_guard.time_environment.periods,
                            &scheduling_environment_guard.time_environment.days,
                            work_order_configurations,
                            material_to_period,
                        )?
                        .build();
                    // Do not guess - use a debugger to understand state
                    drop(scheduling_environment_guard);
                    self.algorithm
                        .parameters
                        .strategic_work_order_parameters
                        .insert(work_order_number, strategic_parameter);
                }

                Ok(StrategicResponseMessage::StateLink)
            }
            StateLink::WorkerEnvironment => {
                let scheduling_environment_guard = self.scheduling_environment.lock().unwrap();
                let strategic_resources =
                    StrategicResources::from((&scheduling_environment_guard, &self.actor_id));
                drop(scheduling_environment_guard);

                self.algorithm
                    .parameters
                    .strategic_capacity
                    .update_resource_capacities(strategic_resources)
                    .expect("Could not update the StrategicResources");

                Ok(StrategicResponseMessage::StateLink)
            }
            StateLink::TimeEnvironment => todo!(),
        }
    }
}
