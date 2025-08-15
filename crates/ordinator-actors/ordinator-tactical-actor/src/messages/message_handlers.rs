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

// TODO [ ]
// Make a TacticalAgent here! I believe that this is the best appraoch. The only
// way that you will find out is by creating the system in the new way you are
// so much out of the water here that getting it to compile and run is the only
// way to consolidate your knowledge.
// TODO [ ] This should be changed.
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

                    // FIX
                    // The solution should also be updated here. Think about how you can make
                    // this generic.
                    // QUESTION
                    // Is this a good way of coding the program? I think that there is common
                    // behavior here that we are going to have to
                    // exploit to make sense of this. You are not creating this in the best
                    // possible way at the moment I think. There is a
                    // better approach for dealing with this.
                    //
                    // You should wrap this up in the `Interface`

                    let tactical_parameter = create_tactical_parameter(
                        work_order,
                        start_days_for_activities,
                        work_order_configurations,
                    )?;

                    // It is only the algorithm that can modify parameters. Not the the Actor
                    // directly you should fix this issue soon. What
                    // about the code. You should make the interface
                    // here for interacting with the algorithm.
                    self.algorithm
                        .parameters
                        .tactical_work_orders
                        .insert(work_order_number, tactical_parameter);

                    // You update the `solution::work_order` but not the `solution::loading`
                    // this is an issue. But the more important question is whether this
                    // matters in the long run of things. The best approach is to make
                    // the system work correctly with the policy that we have set out to
                    // achieve. What is the best approach for doing this? I think that
                    // we should work on the
                    //
                    // I think that the approach here is that the state_link message should
                    // only update the parameters. And not the Solution itself. Where should
                    // the code then be located that makes the system work correctly with the
                    //
                    // There must be a better way of enforcing this? You need a crystal clear
                    // policy here on how to handle the state changes. I think that you should
                    // for the most part rely
                    //
                    // You could make a trait here
                    // QUESTION [ ] 2025-07-17 Does the StrategicActor touch the `Solution`?
                    // No the Strategic does not touch the solution at all. This is insanely
                    // important to get right. You need to be sure what to do about the code
                    // for it to run correctly.
                    //
                    // User -> Orchestrator -> StateLink -> Actor -> Parameter -> Solution
                    //
                    // I think that this should be the flow. At the moment you have
                    //
                    // User -> Orchestrator -> StateLink -> Actor -> Parameter
                    //                                            -> Solution
                    //
                    // What should be done now? I think that the best approach is
                    // to remove this below. And then the force schedule function
                    // should handle this. The issue here is that I am not sure what
                    // will lead to the right outcome.
                    //
                    // You should delete this. And then make it unschedule the work_order
                    // correctly elsewhere.
                    //
                    // This is the goal to reach now.
                    // TODO [ ] 2025-07-17 make the TacticalActor unschedule correctly
                    // based on `Parameter` values and updates.
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

                // The issue here is that `from` does not consume the value. But instead work
                // with the reference.
                let tactical_resources =
                    TacticalResources::from((&scheduling_environment_guard, &self.actor_id));
                drop(scheduling_environment_guard);

                self.algorithm
                    .parameters
                    .tactical_capacity
                    .update_resources(tactical_resources);

                // TODO [ ]
                // Turn this into a JSON
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
