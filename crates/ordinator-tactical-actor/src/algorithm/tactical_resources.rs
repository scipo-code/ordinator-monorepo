use std::collections::HashMap;

use anyhow::Context;
use anyhow::Result;
use colored::Colorize;
use ordinator_actor_core::traits::ActorLinkToSchedulingEnvironment;
use ordinator_scheduling_environment::time_environment::day::Day;
use ordinator_scheduling_environment::time_environment::day::Days;
use ordinator_scheduling_environment::time_environment::period::Period;
use ordinator_scheduling_environment::work_order::operation::Work;
use ordinator_scheduling_environment::worker_environment::resources::ActorCompositeId;
use ordinator_scheduling_environment::worker_environment::resources::Skill;
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
            let average_hours_per_day = self.resources.values().map(|day| {
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

    pub fn new_from_data(resources: Vec<Skill>, tactical_days: Vec<Day>, load: Work) -> Self
    {
        let days_template = vec![load; tactical_days.len()];
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
            .with_context(|| "The resources between the strategic and the tactical should always correspond, unless that the tactical has not been initialized yet".to_string())?
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

impl<'a> From<(&ActorLinkToSchedulingEnvironment<'a>, &ActorCompositeId)> for ProjectResources
{
    fn from(value: (&ActorLinkToSchedulingEnvironment<'a>, &ActorCompositeId)) -> Self
    {
        // TODO: Move hours_per_day to configuration
        let _hours_per_day = 6.0;

        let gradual_reduction = |i: usize| -> f64 {
            match i {
                0..=13 => 1.0,
                14..=27 => 1.0,
                _ => 1.0,
            }
        };

        // FIXME: Support multi-skill resources; currently only handles single skill
        let mut tactical_resources_inner = HashMap::<Skill, Days>::new();
        for operational_configuration_all in value
            .0
            .worker_environment
            .actor_specification
            .get(value.1.asset())
            .expect("Mising actor for the asset")
            .operational()
            .iter()
        {
            for (i, _) in value.0.time_environment.days.iter().enumerate() {
                let resource_periods = tactical_resources_inner
                    // FIXME: Logic error when comparing with WeeklyAgent
                    .entry(
                        operational_configuration_all
                            .1
                            .operational_configuration
                            .resources
                            .iter()
                            // ISSUE #000 - add multi-skill to tactical.
                            .next()
                            .cloned()
                            .unwrap(),
                    )
                    .or_insert_with(|| {
                        Days::zero_from_existing(&Days {
                            days: value
                                .0
                                .time_environment
                                .days
                                .clone()
                                .into_iter()
                                .map(|_| Work::from(0.0))
                                .collect(),
                        })
                    });

                resource_periods.days[i] += Work::from(
                    operational_configuration_all.1.hours_per_day * gradual_reduction(i),
                );
            }
        }
        ProjectResources::new(tactical_resources_inner)
    }
}
