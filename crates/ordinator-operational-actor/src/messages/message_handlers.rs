use std::fmt::Debug;

use anyhow::Context;
use anyhow::Result;
use ordinator_actor_core::Actor;
use ordinator_orchestrator_actor_traits::CommandHandler;
use ordinator_orchestrator_actor_traits::StateLink;
use ordinator_orchestrator_actor_traits::DailyInterface;
use ordinator_orchestrator_actor_traits::SystemSolutions;
use tracing::Level;
use tracing::event;

use super::OperationalRequestMessage;
use super::OperationalResponseMessage;
use super::requests::OperationalSchedulingRequest;
use super::responses::OperationalResponseStatus;
use crate::algorithm::OperationalAlgorithm;
use crate::algorithm::operational_parameter::OperationalParameter;
use crate::algorithm::operational_solution::OperationalSolution;

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
                    .expect("SchedulingEnvironment Mutex could not be acquired.");

                for work_order_number in changed_work_orders {
                    let work_order = locked_scheduling_environment
                        .work_orders
                        .inner
                        .get(&work_order_number)
                        .unwrap();

                    for activity_number in work_order.activity_numbers() {
                        let operational_parameter = match OperationalParameter::new(
                            work_order
                                .operation_work_remaining(activity_number)
                                .context("operation_work_remaining does not exist")?,
                            work_order
                                .operation_preparation(activity_number)
                                .context("operation_preparation does not exist")?,
                        ) {
                            Some(operational_parameter) => operational_parameter,
                            None => continue,
                        };

                        self.algorithm
                            .parameters
                            .work_order_parameters
                            .insert((work_order_number, activity_number), operational_parameter);
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
                // TODO: Include detailed debugging format if error occurs
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

                // Business types must remain distinct from algorithm types.
                // Do not use `OperationalResponseStatus` for algorithm types.
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

                        // TODO: Remove non-functional code and prioritize operational implementation
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
