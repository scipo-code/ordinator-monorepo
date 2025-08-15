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
use ordinator_scheduling_environment::worker_environment::resources::Resources;
use serde::Deserialize;
use serde::Serialize;

use super::DayIndex;

#[derive(Eq, PartialEq, Default, Serialize, Deserialize, Clone)]
pub struct TacticalResources
{
    pub resources: HashMap<Resources, Days>,
}

impl std::fmt::Debug for TacticalResources
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

            // Do you want to call it?
            //
            let mut days_loading = vec![Work::from(0.0); days.days.len()];
            let average_hours_per_day = self.resources.values().map(|day| {
                days_loading.iter_mut().enumerate().for_each(|f| {
                    *f.1 += day
                        .days
                        .get(f.0)
                        .expect("This should neven happen. Look at the implementation above")
                })
            });
            write!(
                f,
                "{}",
                format!(
                    "TacticalResources: \nDays: {number_of_days}\nResources: {resources}\nAverage hours per day: {average_hours_per_day:#?}"
                ).bright_blue()
            )
        } else {
            f.debug_struct("TacticalResources")
                .field("resources", &self.resources)
                .finish()
        }
    }
}
impl TacticalResources
{
    pub fn new(resources: HashMap<Resources, Days>) -> Self
    {
        TacticalResources { resources }
    }

    // This is a horrible data structure,
    pub fn get_resource(&self, resource: &Resources, day: DayIndex) -> Result<&Work>
    {
        self.resources
            .get(resource)
            .with_context(|| format!("Resource not present {resource}"))?
            .days
            .get(day)
            .with_context(|| format!("Day not present {day}"))
    }

    // This is a horrible data structure,
    pub fn get_resource_mut(&mut self, resource: &Resources, day: DayIndex) -> Result<&mut Work>
    {
        self.resources
            .get_mut(resource)
            .with_context(|| format!("Resource not present {resource}"))?
            .days
            .get_mut(day)
            .with_context(|| format!("Day not present {day}"))
    }

    pub fn new_from_data(resources: Vec<Resources>, tactical_days: Vec<Day>, load: Work) -> Self
    {
        let days_template = vec![load; tactical_days.len()];
        let resource_capacity = resources
            .into_iter()
            .map(|resource| {
                let days = days_template.clone();
                (resource, Days { days })
            })
            .collect::<HashMap<_, _>>();

        TacticalResources::new(resource_capacity)
    }

    pub fn update_resources(&mut self, resources: Self)
    {
        for resource in resources.resources {
            self.resources.get_mut(&resource.0).unwrap().days = resource.1.days.to_vec();
        }
    }

    pub fn determine_period_load(
        &self,
        days: &[Day],
        resource: &Resources,
        period: &Period,
    ) -> Result<Work>
    {
        let work = &self
            .resources
            .get(resource)
            .with_context(|| "The resources between the strategic and the tactical should always correspond, unless that the tactical has not been initialized yet".to_string())?
            .days;

        Ok(work
            .iter()
            .enumerate()
            // How should we handle this? The goal is to connect a given period to a set of daily
            // indices. You should do this first I think? Yes?
            .filter(|(index, _)| period.contains_date(days[*index].0))
            // This is where you have to think about the architecture of the code.
            .map(|(_, work)| work)
            .fold(Work::from(0.0), |acc, work| &acc + work))
    }
}

// Is this the correct way to think about the different things? Yes
// let the caller decide
impl<'a> From<(&ActorLinkToSchedulingEnvironment<'a>, &ActorCompositeId)> for TacticalResources
{
    fn from(value: (&ActorLinkToSchedulingEnvironment<'a>, &ActorCompositeId)) -> Self
    {
        // TODO [ ]
        // Move this out of the code and into `configuration`
        let _hours_per_day = 6.0;

        let gradual_reduction = |i: usize| -> f64 {
            match i {
                0..=13 => 1.0,
                14..=27 => 1.0,
                _ => 1.0,
            }
        };

        // WARN
        // Should this be multi skill?
        // This was always wrong. You should never have to make the system function
        // in that way.
        // Should you simply move the Everything? Yes
        let mut tactical_resources_inner = HashMap::<Resources, Days>::new();
        for operational_configuration_all in value
            .0
            .worker_environment
            .actor_specification
            .get(value.1.asset())
            .expect("Mising actor for the asset")
            .operational()
            .iter()
        {
            // There is an error here! You are moving slow on this. You should take a small
            // break and then fix this. After that I think that you should start
            // thinking about what we should do next.
            for (i, _) in value.0.time_environment.days.iter().enumerate() {
                let resource_periods = tactical_resources_inner
                    // FIX
                    // WARN
                    // There is a logic error here. If we want to compare with the
                    // `StrategicAgent`.
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
        TacticalResources::new(tactical_resources_inner)
    }
}
