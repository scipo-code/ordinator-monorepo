use std::collections::HashMap;
use std::sync::MutexGuard;

use anyhow::Context;
use anyhow::Result;
use colored::Colorize;
use ordinator_scheduling_environment::time_environment::day::Day;
use ordinator_scheduling_environment::time_environment::day::Days;
use ordinator_scheduling_environment::time_environment::period::Period;
use ordinator_scheduling_environment::work_order::operation::Work;
use ordinator_scheduling_environment::worker_environment::resources::Skill;
use ordinator_scheduling_hypergraph::schedule_graph::SchedulingHypergraph;
use serde::Deserialize;
use serde::Serialize;

use super::DayIndex;

#[derive(Eq, PartialEq, Default, Serialize, Deserialize, Clone)]
pub struct ProjectResources
{
    pub resources: HashMap<Skill, Days>,
}

impl std::fmt::Debug for ProjectResources
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        if f.alternate() {
            let resources = self.resources.len();

            let days = self
                .resources
                .values()
                .next()
                .cloned()
                .unwrap_or(Days::new(vec![]));

            let number_of_days = days.days.len();

            let mut days_loading = vec![Work::from(0.0); days.days.len()];
            let _average_hours_per_day = self.resources.values().map(|day| {
                days_loading.iter_mut().enumerate().for_each(|f| {
                    *f.1 += day
                        .days
                        .get(f.0)
                        .expect("Day index should always be valid for initialized resources")
                })
            });
            write!(
                f,
                "{}",
                format!("ProjectResources: \nDays: {number_of_days}\nTechnicians: {resources}")
                    .bright_blue()
            )
        } else {
            f.debug_struct("ProjectResources")
                .field("resources", &self.resources)
                .finish()
        }
    }
}
impl ProjectResources
{
    pub fn new(resources: HashMap<Skill, Days>) -> Self
    {
        ProjectResources { resources }
    }

    pub fn get_resource(&self, resource: &Skill, day: DayIndex) -> Result<&Work>
    {
        self.resources
            .get(resource)
            .with_context(|| format!("Resource not present {resource}"))?
            .days
            .get(day)
            .with_context(|| format!("Day not present {day}"))
    }

    pub fn get_resource_mut(&mut self, resource: &Skill, day: DayIndex) -> Result<&mut Work>
    {
        self.resources
            .get_mut(resource)
            .with_context(|| format!("Resource not present {resource}"))?
            .days
            .get_mut(day)
            .with_context(|| format!("Day not present {day}"))
    }

    pub fn new_from_data(resources: Vec<Skill>, project_days: Vec<Day>, load: Work) -> Self
    {
        let days_template = vec![load; project_days.len()];
        let resource_capacity = resources
            .into_iter()
            .map(|resource| {
                let days = days_template.clone();
                (resource, Days { days })
            })
            .collect::<HashMap<_, _>>();

        ProjectResources::new(resource_capacity)
    }

    pub fn update_resources(&mut self, resources: Self)
    {
        for resource in resources.resources {
            self.resources.get_mut(&resource.0).unwrap().days = resource.1.days.to_vec();
        }
    }

    pub fn determine_period_load(&self, resource: &Skill, period: &Period) -> Result<Work>
    {
        let days = &self
            .resources
            .get(resource)
            .with_context(|| "The resources between the weekly and the project should always correspond, unless that the project has not been initialized yet".to_string())?
            .days;

        Ok(days
            .iter()
            .enumerate()
            // Filter to days within the period
            .filter(|(index, _)| period.day_indices.contains(&(*index as u64)))
            .map(|(_, work)| work)
            .fold(Work::from(0.0), |acc, work| &acc + work))
    }

    pub(crate) fn total_hours(&self) -> Work
    {
        self.resources
            .values()
            .fold(Work::from(0.0), |mut acc, days| {
                acc += days.days.iter().cloned().sum::<Work>();
                acc
            })
    }
}

impl ProjectResources
{
    pub fn from_scheduling_hypergraph(
        scheduling_hypergraph: &MutexGuard<SchedulingHypergraph>,
        project_days: &[Day],
    ) -> Self
    {
        let weekly_view = scheduling_hypergraph.extract_weekly_view();

        let hours_per_day = 6.0;
        let num_days = project_days.len();

        // Build capacity: for each technician, contribute hours_per_day to each
        // of their skills for each available date that falls within project_days
        let mut project_resources_inner = HashMap::<Skill, Days>::new();

        for technician in weekly_view.technicians.values() {
            for &skill in &technician.skills {
                let days_entry = project_resources_inner
                    .entry(skill)
                    .or_insert_with(|| Days::new(vec![Work::from(0.0); num_days]));

                for (i, day) in project_days.iter().enumerate() {
                    if technician.available_dates.contains(&day.date) {
                        days_entry.days[i] += Work::from(hours_per_day);
                    }
                }
            }
        }

        ProjectResources::new(project_resources_inner)
    }
}
