use std::fmt::Debug;

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
                let weekly_view = {
                    let scheduling_environment_guard = self.scheduling_environment.lock().unwrap();
                    scheduling_environment_guard.extract_weekly_view()
                };

                for work_order_number in changed_work_orders {
                    if let Some(wo_view) = weekly_view.work_orders.get(&work_order_number) {
                        for activity in &wo_view.activities {
                            let daily_parameter = DailyParameter::new(
                                activity.required_skill,
                                activity.number_of_people,
                                activity.work_remaining,
                            );
                            self.algorithm.parameters.insert_daily_parameter(
                                &(work_order_number, activity.activity_number),
                                daily_parameter,
                            )
                        }
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
