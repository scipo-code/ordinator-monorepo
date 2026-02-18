use std::collections::BTreeMap;

use chrono::NaiveDate;
use ordinator_orchestrator_actor_traits::ProjectInterface;
use ordinator_orchestrator_actor_traits::WhereIsWorkOrder::NotScheduled;
use ordinator_orchestrator_actor_traits::WhereIsWorkOrder::Weekly;
use ordinator_orchestrator_actor_traits::WhereIsWorkOrder::Project;
use ordinator_scheduling_environment::time_environment::period::Period;
use ordinator_scheduling_environment::work_order::WorkOrder;
use ordinator_scheduling_environment::work_order::WorkOrderActivity;
use ordinator_scheduling_environment::work_order::WorkOrderNumber;
use ordinator_scheduling_environment::work_order::operation::Work;
use ordinator_scheduling_environment::worker_environment::resources::Skill;

use super::tactical_solution::ProjectSolution;

impl ProjectInterface for ProjectSolution
{
    fn start_and_finish_dates(
        &self,
        work_order_activity: &WorkOrderActivity,
    ) -> Option<(&NaiveDate, &NaiveDate)>
    {
        let activities = self.tactical_work_orders.0.get(&work_order_activity.0)?;
        let scheduled_days = match &activities {
            Weekly(_) => return None,
            Project(value) => &value.0.get(&work_order_activity.1).unwrap().scheduled,
            NotScheduled => return None,
        };

        let start = &scheduled_days.first().unwrap().0.date;
        let end = &scheduled_days.last().unwrap().0.date;

        Some((start, end))
    }

    fn tactical_period<'a>(
        &self,
        _work_order_number: &WorkOrderNumber,
        periods: &'a [Period],
    ) -> Option<&'a Period>
    {
        match self.tactical_work_orders.0.get(_work_order_number) {
            Some(c) => match c {
                Weekly(_) => None,
                Project(wo) => {
                    let first_activity = wo.0.first_key_value();
                    let first_date = first_activity?.1.scheduled.first()?.0.date;
                    Some(WorkOrder::date_to_period(periods, &first_date))
                }
                NotScheduled => None,
            },
            None => None,
        }
    }

    fn all_scheduled_tasks(
        &self,
    ) -> std::collections::HashMap<
        WorkOrderNumber,
        std::collections::BTreeMap<
            ordinator_scheduling_environment::work_order::operation::ActivityNumber,
            ordinator_scheduling_environment::time_environment::day::Day,
        >,
    >
    {
        self
            .tactical_work_orders
            .0
            .iter()
            .clone()
            .map(|(won, whe_opt)| match whe_opt {
                Weekly(_) => (won, None),
                Project(value) => (won, Some(value)),
                NotScheduled => (won, None),
            })
            .filter(|e| e.1.is_some())
            .map(|(won, e)| {
                (
                    *won,
                    e.unwrap()
                        .0
                        .clone()
                        .iter()
                        .map(|e| (*e.0, e.1.scheduled.first().as_ref().unwrap().0.clone()))
                        .collect::<BTreeMap<_, _>>(),
                )
            })
            .collect()
    }

    fn tactical_loadings(&self) -> BTreeMap<Skill, Vec<Work>>
    {
        self.tactical_loadings
            .resources
            .iter()
            .map(|e| (*e.0, e.1.days.clone()))
            .collect()
    }
}
