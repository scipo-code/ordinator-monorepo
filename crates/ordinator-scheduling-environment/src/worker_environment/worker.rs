use std::fmt::Debug;
use std::fmt::Formatter;
use std::fmt::{self};

use serde::Deserialize;
use serde::Serialize;

use super::availability::Availability;
use crate::worker_environment::resources::Skill;

#[derive(Serialize, Deserialize)]
enum AssignedOrder
{
    OrderInt(i32),
    None,
}

#[derive(Serialize, Deserialize)]
enum AssignedActivity
{
    ActivityInt(i32),
    None,
}

#[derive(Serialize, Deserialize)]
enum AssignedTime
{
    TimeFloat(f64),
    None,
}

#[derive(Serialize, Deserialize)]
struct AssignedWork
{
    order: AssignedOrder,
    activity: AssignedActivity,
    time: AssignedTime,
}

#[derive(Serialize, Deserialize)]
pub struct Technician
{
    name: String,
    id_worker: i32,
    capacity: f64,
    trait_: String,
    skills: Vec<Skill>,
    availabilities: Vec<Availability>,
    assigned_activities: Vec<AssignedWork>,
}

impl Technician
{
    pub fn id(&self) -> usize
    {
        self.id_worker as usize
    }

    pub fn skills(&self) -> &[Skill]
    {
        &self.skills
    }

    pub fn builder(id: usize) -> TechnicianBuilder
    {
        TechnicianBuilder {
            id: id as i32,
            skills: vec![],
            availabilities: vec![],
        }
    }
}

pub struct TechnicianBuilder
{
    id: i32,
    skills: Vec<Skill>,
    availabilities: Vec<Availability>,
}

impl TechnicianBuilder
{
    pub fn add_skill(mut self, skill: Skill) -> Self
    {
        self.skills.push(skill);
        self
    }

    pub fn add_availability(
        mut self,
        start: chrono::NaiveDateTime,
        end: chrono::NaiveDateTime,
    ) -> anyhow::Result<Self>
    {
        self.availabilities
            .push(Availability::from_naive(start, end));
        Ok(self)
    }

    pub fn build(self) -> Technician
    {
        Technician {
            name: format!("Technician-{}", self.id),
            id_worker: self.id,
            capacity: 1.0,
            trait_: String::new(),
            skills: self.skills,
            availabilities: self.availabilities,
            assigned_activities: vec![],
        }
    }
}

impl Debug for Technician
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
    {
        f.debug_struct("Worker")
            .field("name", &self.name)
            .field("id", &self.id_worker)
            .field("capacity", &self.capacity)
            .field("trait_", &self.trait_)
            .field("availabilities", &self.availabilities.len())
            .field("assigned_activities", &self.assigned_activities.len())
            .finish()
    }
}
