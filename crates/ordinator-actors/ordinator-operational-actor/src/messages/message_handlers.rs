use std::fmt::Debug;

use anyhow::Result;
use ordinator_actor_core::Actor;
use ordinator_orchestrator_actor_traits::CommandHandler;
use ordinator_orchestrator_actor_traits::StateLink;
use ordinator_orchestrator_actor_traits::SupervisorInterface;
use ordinator_orchestrator_actor_traits::SystemSolutions;
use tracing::event;
use tracing::Level;

use super::requests::OperationalSchedulingRequest;
use super::responses::OperationalResponseStatus;
use super::OperationalRequestMessage;
use super::OperationalResponseMessage;
use crate::algorithm::operational_parameter::OperationalParameter;
use crate::algorithm::operational_solution::OperationalSolution;
use crate::algorithm::OperationalAlgorithm;
// Was this actually needed? I am not really sure here I believe that
// the best approach is to make something.
impl<Ss> CommandHandler<OperationalRequestMessage, OperationalResponseMessage>
    for Actor<OperationalRequestMessage, OperationalResponseMessage, OperationalAlgorithm<Ss>>
where
    Ss: SystemSolutions<Operational = OperationalSolution> + Debug,
{
    fn handle_state_link(&mut self, state_link: StateLink) -> Result<OperationalResponseMessage>
    {
        event!(
            Level::INFO,
            self.algorithm.operational_parameters =
                self.algorithm.parameters.work_order_parameters.len()
        );
        match state_link {
            StateLink::WorkOrders(changed_work_orders) => {
                event!(target: "business_events", Level::ERROR, unhandled_work_orders = ?changed_work_orders);
                let locked_scheduling_environment = self
                    .scheduling_environment
                    .lock()
                    .expect("SchedulignEnvironment Mutex could not be acquired.");

                for work_order_number in changed_work_orders {
                    let work_order = locked_scheduling_environment
                        .work_orders
                        .inner
                        .get(&work_order_number)
                        .unwrap();

                    for (activity_number, operation) in work_order.operations.0.iter() {
                        let operational_parameter = match OperationalParameter::new(
                            operation.operation_info.work_remaining,
                            operation.operation_analytic.preparation_time,
                        ) {
                            Some(operational_parameter) => operational_parameter,
                            None => continue,
                        };

                        self.algorithm
                            .parameters
                            .work_order_parameters
                            .insert((work_order_number, *activity_number), operational_parameter);
                    }
                }

                Ok(OperationalResponseMessage::Success)
            }
            StateLink::WorkerEnvironment => todo!(),
            StateLink::TimeEnvironment => todo!(),
        }
    }

    fn handle_request_message(
        &mut self,
        request: OperationalRequestMessage,
    ) -> Result<OperationalResponseMessage>
    {
        match request {
            ordinator_actor_core::RequestMessage::Status(_) => {
                // WARN DEBUG: This should be included if you get an error
                //     format!(
                //         "ID: {}, traits: {}, Objective: {:?}",
                //         self.agent_id.0,
                //         self.agent_id
                //             .1
                //             .iter()
                //             .map(|resource| resource.to_string())
                //             .collect::<Vec<String>>()
                //             .join(", "),
                //         self.algorithm
                //             .operational_solution
                //             .objective_value
                //     )
                // }
                let (assign, assess, unassign): (u64, u64, u64) = self
                    .algorithm
                    .loaded_system_solution
                    .supervisor_actor_solutions()?
                    .count_delegate_types(&self.actor_id);

                // Remember that the business types should not be the same type as the
                // algorithm types. That is crucial to understand in all this.
                // These should not have the `OperationalResponseStatus`
                // QUESTION
                // Should the `OperationalObjectiveValue` be shareable? No I do not think so.
                let operational_response_status = OperationalResponseStatus::new(
                    self.actor_id.clone(),
                    assign,
                    assess,
                    unassign,
                    self.algorithm.solution.objective_value,
                );
                Ok(OperationalResponseMessage::Status(
                    operational_response_status,
                ))
            }
            ordinator_actor_core::RequestMessage::Scheduling(operational_scheduling_request) => {
                match operational_scheduling_request {
                    OperationalSchedulingRequest::OperationalIds => todo!(),
                    OperationalSchedulingRequest::OperationalState(_) => {
                        // let mut json_assignments_events: Vec<ApiAssignmentEvents> = vec![];

                        // I think that you should starte removing code that does not really
                        // work here. You have to make something operational fast.
                        // for (work_order_activity, operational_solution) in
                        //     &self.algorithm.solution.scheduled_work_order_activities
                        // {
                        //     let mut json_assignments = vec![];
                        //     for assignment in &operational_solution.assignments {
                        //         // ApiAssignment is an API type not a business type, so where
                        //         // should it go in the code?
                        //         let json_assignment = ApiAssignment::new(
                        //             assignment.operational_events,
                        //             assignment.start,
                        //             assignment.finish,
                        //         );
                        //         json_assignments.push(json_assignment);
                        //     }

                        //     let event_info = EventInfo::new(Some(*work_order_activity));
                        //     let json_assignment_event =
                        //         ApiAssignmentEvents::new(event_info, json_assignments);
                        //     json_assignments_events.push(json_assignment_event);
                        // }

                        todo!()
                        // let operational_scheduling_response =
                        //     OperationalSchedulingResponse::EventList(json_assignments_events);
                        // Ok(OperationalResponseMessage::Scheduling(
                        //     operational_scheduling_response,
                        // ))
                    }
                }
            }
            ordinator_actor_core::RequestMessage::Resource(_) => todo!(),
            ordinator_actor_core::RequestMessage::Time(_) => todo!(),
            ordinator_actor_core::RequestMessage::SchedulingEnvironment(_) => todo!(),
            ordinator_actor_core::RequestMessage::Update => todo!(),
        }
    }
}
