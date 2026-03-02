use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::MutexGuard;

use anyhow::Context;
use anyhow::Result;
use ordinator_actor_core::algorithm::LoadOperation;
use ordinator_scheduling_environment::time_environment::period::Period;
use ordinator_scheduling_environment::work_order::operation::Work;
use ordinator_scheduling_environment::worker_environment::OperationalId;
use ordinator_scheduling_environment::worker_environment::resources::Skill;
use ordinator_scheduling_hypergraph::schedule_graph::SchedulingHypergraph;
use serde::Deserialize;
use serde::Serialize;

#[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct WeeklyResources(pub HashMap<Period, HashMap<Skill, Work>>);

impl WeeklyResources
{
    pub fn from_scheduling_hypergraph(
        scheduling_hypergraph: &MutexGuard<SchedulingHypergraph>,
    ) -> Self
    {
        let weekly_view = scheduling_hypergraph.extract_weekly_view();

        let mut weekly_resources_inner = HashMap::<Period, HashMap<Skill, Work>>::new();

        for period in &weekly_view.periods {
            let mut skill_hours_map: HashMap<Skill, Work> = HashMap::new();

            for technician in weekly_view.technicians.values() {
                let days_in_period = technician
                    .available_dates
                    .iter()
                    .filter(|date| period.contains_date(**date))
                    .count();

                if days_in_period == 0 {
                    continue;
                }

                let work_contribution = Work::from(6.0 * days_in_period as f64);

                for &skill in &technician.skills {
                    *skill_hours_map.entry(skill).or_insert(Work::from(0.0)) += work_contribution;
                }
            }

            weekly_resources_inner.insert(period.clone(), skill_hours_map);
        }

        WeeklyResources(weekly_resources_inner)
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug, Default)]
pub struct OperationalResource
{
    pub id: OperationalId,
    pub total_hours: Work,
    pub skill_hours: HashMap<Skill, Work>,
}

impl OperationalResource
{
    pub fn new(id: &str, total_hours: Work, skills: HashSet<Skill>) -> Self
    {
        let skill_hours: HashMap<Skill, Work> =
            skills.iter().map(|ski| (*ski, total_hours)).collect();

        Self {
            id: id.to_string(),
            total_hours,
            skill_hours,
        }
    }
}

impl WeeklyResources
{
    pub fn assert_well_shaped_resources(&self) -> Result<()>
    {
        Ok(())
    }

    pub fn insert_skill_work(&mut self, period: Period, skill: Skill, work: Work)
    {
        self.0.entry(period).or_default().insert(skill, work);
    }
}

impl WeeklyResources
{
    pub fn new(resources: HashMap<Period, HashMap<Skill, Work>>) -> Self
    {
        Self(resources)
    }

    pub fn update_load(
        &mut self,
        period: &Period,
        resource: Skill,
        load: Work,
        load_operation: LoadOperation,
    )
    {
        let skill_map = self.0.entry(period.clone()).or_default();
        match load_operation {
            LoadOperation::Add => {
                *skill_map.entry(resource).or_insert(Work::from(0.0)) += load;
            }
            LoadOperation::Sub => {
                *skill_map.entry(resource).or_insert(Work::from(0.0)) -= load;
            }
        }
    }

    pub fn update_resource_capacities(&mut self, resources: Self) -> Result<()>
    {
        for (period, skill_map) in &resources.0 {
            for (skill, work) in skill_map {
                self.0
                    .entry(period.clone())
                    .or_default()
                    .entry(*skill)
                    .and_modify(|existing| *existing = *work)
                    .or_insert(*work);
            }
        }
        Ok(())
    }

    pub fn initialize_resource_loadings(&mut self, resources: Self)
    {
        for (period, skill_map) in resources.0 {
            for (skill, _work) in skill_map {
                self.0
                    .entry(period.clone())
                    .or_default()
                    .entry(skill)
                    .and_modify(|existing| *existing = Work::from(0.0))
                    .or_insert(Work::from(0.0));
            }
        }
    }

    pub fn aggregated_capacity_by_period_and_resource(
        &self,
        period: &Period,
        resource: &Skill,
    ) -> Result<Work>
    {
        Ok(*self
            .0
            .get(period)
            .with_context(|| {
                format!(
                    "{} not found in {:?}",
                    period,
                    std::any::type_name::<WeeklyResources>()
                )
            })?
            .get(resource)
            .unwrap_or(&Work::from(0.0)))
    }
}
