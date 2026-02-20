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

use super::DailyRequestMessage;
use super::DailyResponseMessage;
use crate::algorithm::DailyAlgorithm;
use crate::algorithm::daily_parameters::DailyParameter;
use crate::algorithm::daily_parameters::DailyParameters;
use crate::algorithm::daily_solution::DailySolution;
use crate::messages::responses::DailyResponseScheduling;
use crate::messages::responses::DailyResponseStatus;

impl<Ss> CommandHandler<DailyRequestMessage, DailyResponseMessage>
    for Actor<DailyRequestMessage, DailyResponseMessage, DailyAlgorithm<Ss>>
where
    Ss: SystemSolutions<Daily = DailySolution> + Debug,
{
    fn handle_state_link(&mut self, state_link: StateLink) -> Result<DailyResponseMessage>
    {
        match state_link {
            StateLink::WorkOrders(changed_work_orders) => {
                // TODO: Architectural concern - self holds both scheduling_environment and algorithm, which couples concerns
                let work_orders = {
                    let scheduling_environment_guard = self.scheduling_environment.lock().unwrap();

                    scheduling_environment_guard.work_orders.inner.clone()
                };

                for work_order_number in changed_work_orders {
                    let work_order = work_orders.get(&work_order_number).with_context(|| {
                        format!(
                            "{:?} should always be present in {}",
                            work_order_number,
                            std::any::type_name::<DailyParameters>()
                        )
                    })?;

                    for activity_number in work_order.activity_numbers() {
                        let resource = work_order.operation_resource(activity_number)?;
                        let number = work_order.number_of_people(activity_number)?;
                        let work_remaining =
                            work_order.operation_work_remaining(activity_number)?;

                        let daily_parameter =
                            DailyParameter::new(resource, number, work_remaining);
                        self.algorithm.parameters.insert_daily_parameter(
                            &(work_order_number, activity_number),
                            daily_parameter,
                        )
                    }
                }
                Ok(DailyResponseMessage::StateLink)
            }
            StateLink::WorkerEnvironment => Ok(DailyResponseMessage::StateLink),
            StateLink::TimeEnvironment => todo!(),
        }
    }

    fn handle_request_message(
        &mut self,
        daily_request_message: DailyRequestMessage,
    ) -> Result<DailyResponseMessage>
    {
        event!(Level::WARN, "start_of_daily_handler");

        match daily_request_message {
            DailyRequestMessage::Scheduling(_scheduling_message) => Ok(
                DailyResponseMessage::Scheduling(DailyResponseScheduling {}),
            ),
            DailyRequestMessage::Update => {
                bail!(
                    "IMPLEMENT update logic for Daily for Asset: {:?}",
                    self.actor_id.asset()
                );
            }
            DailyRequestMessage::Status(daily_status_message) => {
                event!(Level::WARN, "start of status message initialization");
                tracing::info!(
                    "Received DailyStatusMessage: {:?}",
                    daily_status_message
                );
                let daily_status = DailyResponseStatus {
                    delegated_work_order_activities: self.algorithm.solution.count_unique_woa(),
                    objective: self.algorithm.solution.objective_value.percent(),
                };
                event!(Level::WARN, "after creation of the daily_status");

                Ok(DailyResponseMessage::Status(daily_status))
            }
        }
    }
}
