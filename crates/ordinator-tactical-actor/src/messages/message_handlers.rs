use std::collections::HashMap;
use std::fmt::Debug;

use anyhow::Context;
use anyhow::Result;
use ordinator_actor_core::Actor;
use ordinator_orchestrator_actor_traits::CommandHandler;
use ordinator_orchestrator_actor_traits::StateLink;
use ordinator_orchestrator_actor_traits::SystemSolutions;
use ordinator_scheduling_environment::assignments::AnyAssignment;

use super::TacticalRequestMessage;
use super::TacticalResponseMessage;
use crate::algorithm::TacticalAlgorithm;
use crate::algorithm::tactical_parameters::TacticalParameters;
use crate::algorithm::tactical_parameters::create_tactical_parameter;
use crate::algorithm::tactical_resources::TacticalResources;
use crate::algorithm::tactical_solution::TacticalSolution;

// TODO: Refactor to use TacticalAgent design instead of current implementation
impl<Ss: Debug> CommandHandler<TacticalRequestMessage, TacticalResponseMessage>
    for Actor<TacticalRequestMessage, TacticalResponseMessage, TacticalAlgorithm<Ss>>
where
    Ss: SystemSolutions<Tactical = TacticalSolution>,
{
    fn handle_request_message(
        &mut self,
        tactical_request: TacticalRequestMessage,
    ) -> Result<TacticalResponseMessage>
    {
        match tactical_request {
            TacticalRequestMessage::Status(_tactical_status_message) => {
                // let status_message = self.status().unwrap();
                // Ok(TacticalResponseMessage::Status(status_message))
                todo!()
            }
            TacticalRequestMessage::Scheduling(_tactical_scheduling_message) => {
                todo!()
            }
            TacticalRequestMessage::Resource(_tactical_resources_message) => {
                // let resource_response = self
                //     .update_resources_state(tactical_resources_message)
                //     .unwrap();
                Ok(TacticalResponseMessage::FreeStringResponse(
                    "Implement the Update code here.".to_string(),
                ))
            }
            TacticalRequestMessage::Time(_tactical_time_message) => {
                todo!()
            }
            TacticalRequestMessage::Update => {
                todo!()
                // let locked_scheduling_environment =
                // &self.scheduling_environment.lock().unwrap();
                // let asset = &self.asset;

                // self.algorithm
                //     .create_tactical_parameters(locked_scheduling_environment, asset);
                // Ok(TacticalResponseMessage::Update)
            }
        }
    }

    fn handle_state_link(&mut self, state_link: StateLink) -> Result<TacticalResponseMessage>
    {
        match state_link {
            StateLink::WorkOrders(modified_work_orders) => {
                let scheduling_environment_guard = self.scheduling_environment.lock().unwrap();

                let work_orders = &scheduling_environment_guard.work_orders.inner.clone();
                let work_order_configurations =
                    &scheduling_environment_guard.work_order_policies.clone();

                let assignments: Vec<_> = scheduling_environment_guard
                    .assignments
                    .assignment_for_tactical()
                    .iter()
                    .map(|e| (*e.0, e.1.clone()))
                    .collect();

                drop(scheduling_environment_guard);
                for work_order_number in modified_work_orders {
                    let work_order = work_orders.get(&work_order_number).with_context(|| {
                        format!(
                            "{:?} should always be present in {}",
                            work_order_number,
                            std::any::type_name::<TacticalParameters>()
                        )
                    })?;

                    let start_days_for_activities: HashMap<Option<u64>, AnyAssignment> =
                        assignments
                            .iter()
                            .filter(|e| e.1.work_order_number() == work_order_number)
                            .map(|e| (e.1.activity_number(), e.1.clone()))
                            .collect::<HashMap<_, _>>();

                    // TODO: Make solution updates generic and wrap in Interface trait

                    let tactical_parameter = create_tactical_parameter(
                        work_order,
                        start_days_for_activities,
                        work_order_configurations,
                    )?;

                    // Only the algorithm can modify parameters; create an interface for this
                    self.algorithm
                        .parameters
                        .tactical_work_orders
                        .insert(work_order_number, tactical_parameter);

                    // TODO: Ensure solution state remains consistent with parameter updates.
                    // StateLink should only update parameters, not solution directly.
                    // The Strategic actor does not touch solution, maintain this invariant.
                    self.algorithm
                        .unschedule_specific_work_order(work_order_number)
                        .with_context(|| {
                            format!(
                                "could not unschedule tactical work order:\n{work_order_number:#?}"
                            )
                        })?;
                }
                Ok(TacticalResponseMessage::FreeStringResponse(
                    "Updated StateLink::WorkOrders".to_string(),
                ))
            }
            StateLink::WorkerEnvironment => {
                let scheduling_environment_guard = self.scheduling_environment.lock().unwrap();

                // Convert reference to TacticalResources without consuming the value
                let tactical_resources =
                    TacticalResources::from((&scheduling_environment_guard, &self.actor_id));
                drop(scheduling_environment_guard);

                self.algorithm
                    .parameters
                    .tactical_capacity
                    .update_resources(tactical_resources);

                // TODO: Return JSON response instead of string
                Ok(TacticalResponseMessage::FreeStringResponse(
                    "Updated StateLink::WorkerEnvironment".to_string(),
                ))
            }

            StateLink::TimeEnvironment => {
                todo!()
            }
        }
    }
}

// impl<Ss> TacticalActor<TacticalRequestMessage, TacticalResponseMessage,
// TacticalAlgorithm<Ss>> {
//     fn update_resources_state(
//         &mut self,
//         resource_message: TacticalResourceRequest,
//     ) -> Result<TacticalResourceResponse>
//     {
//         match resource_message {
// TacticalResourceRequest::SetResources(resources) => {
//     // The resources should be initialized together with the Agent itself
//     let mut count = 0;
//     for (resource, days) in resources.resources {
//         for (day, capacity) in days.days {
//             let day: Day = match self
//                 .algorithm
//                 .parameters
//                 .tactical_days
//                 .iter()
//                 .find(|d| **d == day)
//             {
//                 Some(day) => {
//                     count += 1;
//                     day.clone()
//                 }
//                 None => {
//                     bail!("Day not found in the tactical days".to_string(),);
//                 }
//             };

//             *self.algorithm.capacity_mut(&resource, &day) = capacity;
//         }
//     }
//     Ok(TacticalResourceResponse::UpdatedResources(count))
// TacticalResourceRequest::GetLoadings {
//     days_end: _,
//     select_resources: _,
// } => {
//     let loadings = self.algorithm.solution.tactical_loadings.clone();

//     let tactical_response_resources =
// TacticalResourceResponse::Loading(loadings);
//     Ok(tactical_response_resources)
// }
// TacticalResourceRequest::GetCapacities {
//     days_end: _,
//     select_resources: _,
// } => {
//     let capacities = self.algorithm.parameters.tactical_capacity.clone();

//     let tactical_response_resources =
// TacticalResourceResponse::Capacity(capacities);

//     Ok(tactical_response_resources)
// }
// TacticalResourceRequest::GetPercentageLoadings {
//     days_end: _,
//     resources: _,
// } => {
//     let capacities = &self.algorithm.parameters.tactical_capacity;
//     let loadings = &self.algorithm.solution.tactical_loadings;

//     let tactical_response_resources =
//         TacticalResourceResponse::Percentage((capacities.clone(),
// loadings.clone()));     Ok(tactical_response_resources)
// }
//             _ => todo!(),
//         }
//     }
// }
