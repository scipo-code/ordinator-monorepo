use std::collections::HashMap;
use std::fmt::Debug;

use anyhow::Context;
use anyhow::Result;
use ordinator_actor_core::Actor;
use ordinator_orchestrator_actor_traits::CommandHandler;
use ordinator_orchestrator_actor_traits::StateLink;
use ordinator_orchestrator_actor_traits::SystemSolutions;
use ordinator_scheduling_environment::assignments::AnyAssignment;

use super::ProjectRequestMessage;
use super::ProjectResponseMessage;
use crate::algorithm::ProjectAlgorithm;
use crate::algorithm::project_parameters::ProjectParameters;
use crate::algorithm::project_parameters::create_project_parameter;
use crate::algorithm::project_resources::ProjectResources;
use crate::algorithm::project_solution::ProjectSolution;

// TODO: Refactor to use ProjectAgent design instead of current implementation
impl<Ss: Debug> CommandHandler<ProjectRequestMessage, ProjectResponseMessage>
    for Actor<ProjectRequestMessage, ProjectResponseMessage, ProjectAlgorithm<Ss>>
where
    Ss: SystemSolutions<Project = ProjectSolution>,
{
    fn handle_request_message(
        &mut self,
        project_request: ProjectRequestMessage,
    ) -> Result<ProjectResponseMessage>
    {
        match project_request {
            ProjectRequestMessage::Status(_project_status_message) => {
                // let status_message = self.status().unwrap();
                // Ok(ProjectResponseMessage::Status(status_message))
                todo!()
            }
            ProjectRequestMessage::Scheduling(_project_scheduling_message) => {
                todo!()
            }
            ProjectRequestMessage::Resource(_project_resources_message) => {
                // let resource_response = self
                //     .update_resources_state(project_resources_message)
                //     .unwrap();
                Ok(ProjectResponseMessage::FreeStringResponse(
                    "Implement the Update code here.".to_string(),
                ))
            }
            ProjectRequestMessage::Time(_project_time_message) => {
                todo!()
            }
            ProjectRequestMessage::Update => {
                todo!()
                // let locked_scheduling_environment =
                // &self.scheduling_environment.lock().unwrap();
                // let asset = &self.asset;

                // self.algorithm
                //     .create_project_parameters(locked_scheduling_environment, asset);
                // Ok(ProjectResponseMessage::Update)
            }
        }
    }

    fn handle_state_link(&mut self, state_link: StateLink) -> Result<ProjectResponseMessage>
    {
        match state_link {
            StateLink::WorkOrders(modified_work_orders) => {
                let scheduling_environment_guard = self.scheduling_environment.lock().unwrap();

                let work_orders = &scheduling_environment_guard.work_orders.inner.clone();
                let work_order_configurations =
                    &scheduling_environment_guard.work_order_policies.clone();

                let assignments: Vec<_> = scheduling_environment_guard
                    .assignments
                    .assignment_for_project()
                    .iter()
                    .map(|e| (*e.0, e.1.clone()))
                    .collect();

                drop(scheduling_environment_guard);
                for work_order_number in modified_work_orders {
                    let work_order = work_orders.get(&work_order_number).with_context(|| {
                        format!(
                            "{:?} should always be present in {}",
                            work_order_number,
                            std::any::type_name::<ProjectParameters>()
                        )
                    })?;

                    let start_days_for_activities: HashMap<Option<u64>, AnyAssignment> =
                        assignments
                            .iter()
                            .filter(|e| e.1.work_order_number() == work_order_number)
                            .map(|e| (e.1.activity_number(), e.1.clone()))
                            .collect::<HashMap<_, _>>();

                    // TODO: Make solution updates generic and wrap in Interface trait

                    let project_parameter = create_project_parameter(
                        work_order,
                        start_days_for_activities,
                        work_order_configurations,
                    )?;

                    // Only the algorithm can modify parameters; create an interface for this
                    self.algorithm
                        .parameters
                        .project_work_orders
                        .insert(work_order_number, project_parameter);

                    // TODO: Ensure solution state remains consistent with parameter updates.
                    // StateLink should only update parameters, not solution directly.
                    // The Weekly actor does not touch solution, maintain this invariant.
                    self.algorithm
                        .unschedule_specific_work_order(work_order_number)
                        .with_context(|| {
                            format!(
                                "could not unschedule project work order:\n{work_order_number:#?}"
                            )
                        })?;
                }
                Ok(ProjectResponseMessage::FreeStringResponse(
                    "Updated StateLink::WorkOrders".to_string(),
                ))
            }
            StateLink::WorkerEnvironment => {
                let scheduling_environment_guard = self.scheduling_environment.lock().unwrap();

                // Convert reference to ProjectResources without consuming the value
                let project_resources =
                    ProjectResources::from((&scheduling_environment_guard, &self.actor_id));
                drop(scheduling_environment_guard);

                self.algorithm
                    .parameters
                    .project_capacity
                    .update_resources(project_resources);

                // TODO: Return JSON response instead of string
                Ok(ProjectResponseMessage::FreeStringResponse(
                    "Updated StateLink::WorkerEnvironment".to_string(),
                ))
            }

            StateLink::TimeEnvironment => {
                todo!()
            }
        }
    }
}

// impl<Ss> ProjectActor<ProjectRequestMessage, ProjectResponseMessage,
// ProjectAlgorithm<Ss>> {
//     fn update_resources_state(
//         &mut self,
//         resource_message: ProjectResourceRequest,
//     ) -> Result<ProjectResourceResponse>
//     {
//         match resource_message {
// ProjectResourceRequest::SetResources(resources) => {
//     // The resources should be initialized together with the Agent itself
//     let mut count = 0;
//     for (resource, days) in resources.resources {
//         for (day, capacity) in days.days {
//             let day: Day = match self
//                 .algorithm
//                 .parameters
//                 .project_days
//                 .iter()
//                 .find(|d| **d == day)
//             {
//                 Some(day) => {
//                     count += 1;
//                     day.clone()
//                 }
//                 None => {
//                     bail!("Day not found in the project days".to_string(),);
//                 }
//             };

//             *self.algorithm.capacity_mut(&resource, &day) = capacity;
//         }
//     }
//     Ok(ProjectResourceResponse::UpdatedResources(count))
// ProjectResourceRequest::GetLoadings {
//     days_end: _,
//     select_resources: _,
// } => {
//     let loadings = self.algorithm.solution.project_loadings.clone();

//     let project_response_resources =
// ProjectResourceResponse::Loading(loadings);
//     Ok(project_response_resources)
// }
// ProjectResourceRequest::GetCapacities {
//     days_end: _,
//     select_resources: _,
// } => {
//     let capacities = self.algorithm.parameters.project_capacity.clone();

//     let project_response_resources =
// ProjectResourceResponse::Capacity(capacities);

//     Ok(project_response_resources)
// }
// ProjectResourceRequest::GetPercentageLoadings {
//     days_end: _,
//     resources: _,
// } => {
//     let capacities = &self.algorithm.parameters.project_capacity;
//     let loadings = &self.algorithm.solution.project_loadings;

//     let project_response_resources =
//         ProjectResourceResponse::Percentage((capacities.clone(),
// loadings.clone()));     Ok(project_response_resources)
// }
//             _ => todo!(),
//         }
//     }
// }
