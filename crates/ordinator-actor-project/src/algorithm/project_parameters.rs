use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fmt::Display;
use std::fmt::{self};
use std::sync::MutexGuard;

use anyhow::Result;
use chrono::NaiveDate;
use ordinator_orchestrator_actor_traits::Parameters;
use ordinator_scheduling_environment::SchedulingEnvironment;
use ordinator_scheduling_environment::time_environment::day::Day;
use ordinator_scheduling_environment::work_order::ActivityRelation;
use ordinator_scheduling_environment::work_order::WorkOrderNumber;
use ordinator_scheduling_environment::work_order::operation::ActivityNumber;
use ordinator_scheduling_environment::work_order::operation::Work;
use ordinator_scheduling_environment::work_order::operation::operation_info::NumberOfPeople;
use ordinator_scheduling_environment::worker_environment::ProjectOptions;
use ordinator_scheduling_environment::worker_environment::resources::ActorCompositeId;
use ordinator_scheduling_environment::worker_environment::resources::Skill;
use ordinator_scheduling_hypergraph::derive_instances::WeeklyWorkOrderView;
use ordinator_scheduling_hypergraph::schedule_graph::SchedulingHypergraph;
use serde::Serialize;

use super::project_resources::ProjectResources;

#[derive(Debug)]
pub struct ProjectParameters
{
    pub project_work_orders: HashMap<WorkOrderNumber, ProjectParameter>,
    pub project_days: Vec<Day>,
    pub project_capacity: ProjectResources,
}

impl Parameters for ProjectParameters
{
    type Key = WorkOrderNumber;
    type Options = ProjectOptions;

    fn from_scheduling_hypergraph(
        _id: &ActorCompositeId,
        scheduling_hypergraph: &MutexGuard<SchedulingHypergraph>,
        options: &Self::Options,
    ) -> Result<Self>
    {
        let weekly_view = scheduling_hypergraph.extract_weekly_view();
        let operating_time = Work::from(options.work_order_policies.operating_time as f64);

        let project_work_orders: HashMap<WorkOrderNumber, ProjectParameter> = weekly_view
            .work_orders
            .iter()
            .map(|(&won, wo_view)| {
                let project_parameter =
                    create_project_parameter_from_view(won, wo_view, operating_time);
                (won, project_parameter)
            })
            .collect();

        // Derive project days from available dates in the hypergraph
        // Collect all unique dates from technician availability and sort them
        let mut all_dates: Vec<NaiveDate> = weekly_view
            .technicians
            .values()
            .flat_map(|t| t.available_dates.iter().copied())
            .collect();
        all_dates.sort();
        all_dates.dedup();

        let project_days: Vec<Day> = all_dates
            .into_iter()
            .enumerate()
            .map(|(i, date)| Day::new(i, date))
            .collect();

        // Build capacity from hypergraph technician data
        let project_capacity =
            ProjectResources::from_scheduling_hypergraph(scheduling_hypergraph, &project_days);

        Ok(Self {
            project_work_orders,
            project_days,
            project_capacity,
        })
    }

    fn create_and_insert_new_parameter(
        &mut self,
        _key: Self::Key,
        _scheduling_environment: MutexGuard<SchedulingEnvironment>,
    )
    {
        todo!()
    }
}

pub fn create_project_parameter_from_view(
    work_order_number: WorkOrderNumber,
    wo_view: &WeeklyWorkOrderView,
    operating_time: Work,
) -> ProjectParameter
{
    let mut operation_parameters = BTreeMap::new();
    let mut relations = Vec::new();

    for activity in &wo_view.activities {
        let operation_parameter = OperationParameter {
            work_order_number,
            number: activity.number_of_people,
            duration: activity.work_remaining,
            operating_time,
            work_remaining: activity.work_remaining,
            resource: activity.required_skill,
            forced_start_date: None,
        };
        operation_parameters.insert(activity.activity_number, operation_parameter);

        if let Some(relation) = &activity.relation_to_next {
            relations.push(relation.clone());
        }
    }

    // Weight derived from total work remaining
    let weight = wo_view
        .activities
        .iter()
        .map(|a| a.work_remaining.to_f64() as u64)
        .sum::<u64>()
        .max(1);

    ProjectParameter {
        project_operation_parameters: operation_parameters,
        weight,
        relations,
        earliest_allowed_start_date: wo_view.basic_start_date,
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectParameter
{
    pub project_operation_parameters: BTreeMap<ActivityNumber, OperationParameter>,
    // TODO: ISSUE #300 Implement forced_schedule_* in the project actor
    pub weight: u64,
    pub relations: Vec<ActivityRelation>,
    // TODO: Move earliest_allowed_start_date to SchedulingEnvironment
    pub earliest_allowed_start_date: NaiveDate,
}

// TODO: Add earliest_start_day field
#[derive(Clone, Serialize, Debug)]
pub struct OperationParameter
{
    pub work_order_number: WorkOrderNumber,
    // TODO: ISSUE #300 Implement forced_schedule_* in the project actor
    // pub forced_start_day: Option<Day>,
    pub number: NumberOfPeople,
    pub duration: Work,
    pub operating_time: Work,
    pub work_remaining: Work,
    pub resource: Skill,
    pub forced_start_date: Option<Day>,
}

impl Display for OperationParameter
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        write!(
            f,
            "OperationParameters:\n
        {:?}\n
        number: {}\n
        duration: {}\n
        operating_time: {:?}\n
        work_remaining: {}\n
        resource: {}",
            self.work_order_number,
            self.number,
            self.duration,
            self.operating_time,
            self.work_remaining,
            self.resource
        )
    }
}
