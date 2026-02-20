use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fmt::Display;

use anyhow::Context;
use anyhow::Result;
use anyhow::bail;
use colored::Colorize;
use ordinator_orchestrator_actor_traits::Solution;
use ordinator_orchestrator_actor_traits::SolutionState;
use ordinator_orchestrator_actor_traits::SwapSolution;
use ordinator_orchestrator_actor_traits::SystemSolutions;
use ordinator_orchestrator_actor_traits::WhereIsWorkOrder;
use ordinator_scheduling_environment::Percent;
use ordinator_scheduling_environment::time_environment::day::Day;
use ordinator_scheduling_environment::time_environment::day::Days;
use ordinator_scheduling_environment::work_order::WorkOrderActivity;
use ordinator_scheduling_environment::work_order::WorkOrderNumber;
use ordinator_scheduling_environment::work_order::operation::ActivityNumber;
use ordinator_scheduling_environment::work_order::operation::Work;
use ordinator_scheduling_environment::work_order::operation::operation_info::NumberOfPeople;
use ordinator_scheduling_environment::worker_environment::ProjectOptions;
use ordinator_scheduling_environment::worker_environment::resources::ActorCompositeId;
use ordinator_scheduling_environment::worker_environment::resources::Skill;
use serde::Deserialize;
use serde::Serialize;
use tracing::Level;
use tracing::event;
use valuable::Valuable;

use super::project_parameters::ProjectParameters;
use super::project_resources::ProjectResources;

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize, Clone, Valuable)]
pub struct ProjectObjectiveValue
{
    pub objective_value: u64,
    pub urgency: (usize, u64),
    pub resource_penalty: (usize, u64),
    pub percent_scheduled: (usize, Percent),
}

/// Represents a project objective value with multiple optimization criteria (assumes minimization)
impl ProjectObjectiveValue
{
    pub fn new(project_options: &ProjectOptions) -> Self
    {
        Self {
            objective_value: u64::MAX,
            urgency: (project_options.urgency, u64::MAX),
            resource_penalty: (project_options.resource_penalty, u64::MAX),
            percent_scheduled: (usize::MIN, Percent::new(0, 100).unwrap()),
        }
    }

    pub fn aggregate_objectives(&mut self)
    {
        self.objective_value = self.urgency.0 as u64 * self.urgency.1
            + self.resource_penalty.0 as u64 * self.resource_penalty.1;
    }
}

#[derive(PartialEq, Eq, Clone)]
pub struct ProjectSolution
{
    pub(crate) objective_value: ProjectObjectiveValue,
    pub(crate) project_work_orders: ProjectScheduledWorkOrders,
    pub(crate) project_loadings: ProjectResources,
}

impl std::fmt::Debug for ProjectSolution
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        if f.alternate() {
            let project_work_orders = self.project_work_orders.0.len();

            write!(
                f,
                "{}",
                format!(
                    "{:#?}\nnumber of project work orders: {}\n{:#?}",
                    self.objective_value, project_work_orders, self.project_loadings,
                )
                .bright_blue()
            )
        } else {
            f.debug_struct("ProjectSolution")
                .field("objective_value", &self.objective_value)
                .field("project_work_orders", &self.project_work_orders)
                .field("project_loadings", &self.project_loadings)
                .finish()
        }
    }
}

impl Solution for ProjectSolution
{
    type Objective = ProjectObjectiveValue;
    type Parameters = ProjectParameters;

    fn from_parameters(parameters: &Self::Parameters) -> Result<Self>
    {
        let project_loadings_inner: HashMap<Skill, Days> = parameters
            .project_capacity
            .resources
            .iter()
            .map(|(wo, days)| {
                let inner_map = days.days.iter().map(|_| Work::from(0.0)).collect();
                (*wo, Days::new(inner_map))
            })
            .collect();

        event!(target: "developer", Level::INFO, project_capacity = ?parameters.project_capacity);

        let project_scheduled_work_orders_inner: HashMap<_, _> = parameters
            .project_work_orders
            .keys()
            .map(|won| (*won, WhereIsWorkOrder::NotScheduled))
            .collect();

        Ok(Self {
            objective_value: ProjectObjectiveValue::new(&parameters.project_options),
            project_work_orders: ProjectScheduledWorkOrders(project_scheduled_work_orders_inner),
            project_loadings: ProjectResources::new(project_loadings_inner),
        })
    }

    fn update_objective(&mut self, other_objective_value: Self::Objective)
    {
        self.objective_value = other_objective_value;
    }
}

impl<Ss> SwapSolution<Ss> for ProjectSolution
where
    Ss: SystemSolutions<Project = ProjectSolution>,
{
    fn swap(id: &ActorCompositeId, solution: SolutionState<Self>, system_solution: &mut Ss)
    {
        system_solution.project_swap(id, solution);
    }
}

impl ProjectSolution
{
    pub fn project_scheduled_days(
        &self,
        work_order_number: &WorkOrderNumber,
        activity_number: &ActivityNumber,
    ) -> Result<&Vec<(Day, Work)>>
    {
        let project_day = &self
            .project_work_orders
            .0
            .get(work_order_number)
            .with_context(|| {
                format!("WorkOrderNumber: {work_order_number:?} was not present in the project solution")
            })?
            .project_operations()
            .with_context(|| {
                format!("WorkOrderNumber: {work_order_number:?} was not scheduled for the project solution")
            })?
            .0
            .get(activity_number)
            .with_context(|| {
                format!("ActivityNumber: {activity_number:?} was not present in the project solution")
            })?
            .scheduled;

        Ok(project_day)
    }

    pub fn project_insert_work_order(
        &mut self,
        work_order_number: WorkOrderNumber,
        project_scheduled_operations: ProjectScheduledOperations,
    )
    {
        self.project_work_orders.0.insert(
            work_order_number,
            WhereIsWorkOrder::Project(project_scheduled_operations),
        );
    }
}

#[derive(PartialEq, Eq, Debug, Default, Clone)]
pub struct ProjectScheduledWorkOrders(
    pub HashMap<WorkOrderNumber, WhereIsWorkOrder<ProjectScheduledOperations>>,
);

pub trait ProjectWhereIsWorkOrder
{
    fn is_project(&self) -> bool;

    fn project_operations(&self) -> Result<&ProjectScheduledOperations>;
}
impl ProjectWhereIsWorkOrder for WhereIsWorkOrder<ProjectScheduledOperations>
{
    fn is_project(&self) -> bool
    {
        matches!(self, WhereIsWorkOrder::Project(_))
    }

    fn project_operations(&self) -> Result<&ProjectScheduledOperations>
    {
        match self {
            WhereIsWorkOrder::Weekly(_) => bail!(
                "A call to extract the {} was made but received {}",
                std::any::type_name::<ProjectScheduledOperations>(),
                std::any::type_name_of_val(self),
            ),
            WhereIsWorkOrder::Project(project_scheduled_operations) => {
                Ok(project_scheduled_operations)
            }
            WhereIsWorkOrder::NotScheduled => bail!(
                "The work order has not been scheduled yet, you are most likely calling this method before complete initialization"
            ),
        }
    }
}

impl ProjectScheduledWorkOrders
{
    pub fn scheduled_work_orders(&self) -> usize
    {
        self.0
            .iter()
            .filter(|(_won, sch_wo)| sch_wo.is_project())
            .count()
    }
}

#[derive(PartialEq, Eq, Debug, Default, Clone)]
pub struct ProjectScheduledOperations(pub BTreeMap<ActivityNumber, OperationSolution>);

impl ProjectScheduledOperations
{
    pub fn insert_operation_solution(
        &mut self,
        activity: ActivityNumber,
        operation_solution: OperationSolution,
    )
    {
        self.0.insert(activity, operation_solution);
    }
}

impl Display for ProjectScheduledOperations
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        let mut project_operations = self.0.iter().collect::<Vec<_>>();
        project_operations
            .sort_by(|a, b| a.1.work_order_activity.1.cmp(&b.1.work_order_activity.1));

        for operation_solution in project_operations {
            write!(f, "activity: {:#?}", operation_solution.0)?;
            write!(f, "{}", operation_solution.1)?;
        }
        Ok(())
    }
}

#[allow(dead_code)]
pub struct ProjectSolutionBuilder(ProjectSolution);

#[allow(dead_code)]
impl ProjectSolutionBuilder
{
    pub fn with_project_days(
        mut self,
        project_days: HashMap<WorkOrderNumber, WhereIsWorkOrder<ProjectScheduledOperations>>,
    ) -> Self
    {
        self.0.project_work_orders.0 = project_days;
        self
    }

    pub fn build(self) -> ProjectSolution
    {
        ProjectSolution {
            objective_value: self.0.objective_value,
            project_work_orders: self.0.project_work_orders,
            project_loadings: self.0.project_loadings,
        }
    }
}
#[derive(Hash, PartialEq, PartialOrd, Ord, Eq, Clone, Debug, Serialize)]
pub struct OperationSolution
{
    pub scheduled: Vec<(Day, Work)>,
    pub resource: Skill,
    pub number: NumberOfPeople,
    pub work_remaining: Work,
    pub work_order_activity: WorkOrderActivity,
}

impl OperationSolution
{
    pub fn new(
        scheduled: Vec<(Day, Work)>,
        resource: Skill,
        number: NumberOfPeople,
        work_remaining: Work,
        work_order_number: WorkOrderNumber,
        activity_number: ActivityNumber,
    ) -> OperationSolution
    {
        OperationSolution {
            scheduled,
            resource,
            number,
            work_remaining,
            work_order_activity: (work_order_number, activity_number),
        }
    }
}

impl Display for OperationSolution
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        write!(f, "{:?}", self.work_order_activity)?;
        for scheduled in &self.scheduled {
            write!(f, "{} on {}", scheduled.1, scheduled.0)?
        }
        Ok(())
    }
}
