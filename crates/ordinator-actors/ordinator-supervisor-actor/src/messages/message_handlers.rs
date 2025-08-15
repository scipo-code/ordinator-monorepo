use std::fmt::Debug;

use anyhow::Context;
use anyhow::Result;
use anyhow::bail;
use ordinator_actor_core::Actor;
use ordinator_orchestrator_actor_traits::CommandHandler;
use ordinator_orchestrator_actor_traits::StateLink;
use ordinator_orchestrator_actor_traits::SystemSolutions;
use tracing::Level;
use tracing::event;

use super::SupervisorRequestMessage;
use super::SupervisorResponseMessage;
use crate::algorithm::SupervisorAlgorithm;
use crate::algorithm::supervisor_parameters::SupervisorParameter;
use crate::algorithm::supervisor_parameters::SupervisorParameters;
use crate::algorithm::supervisor_solution::SupervisorSolution;
use crate::messages::responses::SupervisorResponseScheduling;
use crate::messages::responses::SupervisorResponseStatus;

// Should you implement on the new ty
impl<Ss> CommandHandler<SupervisorRequestMessage, SupervisorResponseMessage>
    for Actor<SupervisorRequestMessage, SupervisorResponseMessage, SupervisorAlgorithm<Ss>>
where
    Ss: SystemSolutions<Supervisor = SupervisorSolution> + Debug,
{
    fn handle_state_link(&mut self, state_link: StateLink) -> Result<SupervisorResponseMessage>
    {
        match state_link {
            StateLink::WorkOrders(changed_work_orders) => {
                // It is beginning to seem a little horrible that the self. here holds both the
                // `scheduling_environment` and the `algorithm`. There is a
                // couple of issues here relating to how we interact
                // with the algorithm. I
                let work_orders = {
                    let scheduling_environment_guard = self.scheduling_environment.lock().unwrap();

                    scheduling_environment_guard.work_orders.inner.clone()
                };

                for work_order_number in changed_work_orders {
                    let work_order = work_orders.get(&work_order_number).with_context(|| {
                        format!(
                            "{:?} should always be present in {}",
                            work_order_number,
                            std::any::type_name::<SupervisorParameters>()
                        )
                    })?;

                    for activity_number in work_order.activity_numbers() {
                        let resource = work_order.operation_resource(activity_number)?;
                        let number = work_order.number_of_people(activity_number)?;
                        let work_remaining =
                            work_order.operation_work_remaining(activity_number)?;

                        let supervisor_parameter =
                            SupervisorParameter::new(resource, number, work_remaining);
                        self.algorithm.parameters.insert_supervisor_parameter(
                            &(work_order_number, activity_number),
                            supervisor_parameter,
                        )
                    }
                }
                Ok(SupervisorResponseMessage::StateLink)
            }
            StateLink::WorkerEnvironment => Ok(SupervisorResponseMessage::StateLink),
            StateLink::TimeEnvironment => todo!(),
        }
    }

    fn handle_request_message(
        &mut self,
        supervisor_request_message: SupervisorRequestMessage,
    ) -> Result<SupervisorResponseMessage>
    {
        event!(Level::WARN, "start_of_supervisor_handler");

        match supervisor_request_message {
            SupervisorRequestMessage::Scheduling(_scheduling_message) => Ok(
                SupervisorResponseMessage::Scheduling(SupervisorResponseScheduling {}),
            ),
            SupervisorRequestMessage::Update => {
                bail!(
                    "IMPLEMENT update logic for Supervisor for Asset: {:?}",
                    self.actor_id.asset()
                );
            }
            SupervisorRequestMessage::Status(supervisor_status_message) => {
                event!(Level::WARN, "start of status message initialization");
                tracing::info!(
                    "Received SupervisorStatusMessage: {:?}",
                    supervisor_status_message
                );
                let supervisor_status = SupervisorResponseStatus {
                    delegated_work_order_activities: self.algorithm.solution.count_unique_woa(),
                    objective: self.algorithm.solution.objective_value,
                };
                event!(Level::WARN, "after creation of the supervisor_status");

                Ok(SupervisorResponseMessage::Status(supervisor_status))
            }
        }
    }
}
