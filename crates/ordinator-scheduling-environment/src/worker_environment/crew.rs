use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::HashSet;

use serde::Deserialize;
use serde::Serialize;

use super::availability::Availability;
use super::resources::ActorCompositeId;
use super::resources::Resources;
use crate::time_environment::TimeInterval;
use crate::worker_environment::worker::Worker;

// TODO [ ]
// This should go to the `SchedulingEnvironment::worker_environment`
// How should this go about? Do you want to separate the workers
// by Asset? Ideally you would not, but I think that here it is
// a really good idea.
#[derive(Clone, Default, Serialize, Deserialize, Debug)]
pub struct AgentEnvironment
{
    // TODO [ ]
    // Rename these they have a horrible name, they have nothing to do with
    pub operational: HashMap<ActorCompositeId, OperationalConfigurationAll>,
    pub supervisor: HashMap<ActorCompositeId, SupervisorConfigurationAll>,
}

// WARN
// You should never be able to clone this.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OperationalConfigurationAll
{
    pub id: ActorCompositeId,
    pub hours_per_day: f64,
    pub operational_configuration: OperationalConfiguration,
}

impl OperationalConfigurationAll
{
    pub fn new(
        id: ActorCompositeId,
        hours_per_day: f64,
        operational_configuration: OperationalConfiguration,
    ) -> Self
    {
        Self {
            id,
            hours_per_day,
            operational_configuration,
        }
    }
}

// Ahh shit. You have to make them work with the correct data. You have to
// determine right now if you want to make something that. The way that you have
// it have is a little weird You should make a high level decision about how to
// structure the data correct.
//
// Ask God what to do here?
// We should have all the required data in the scheduling environment.
#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct OperationalConfiguration
{
    pub availability: BTreeSet<Availability>,
    pub break_interval: TimeInterval,
    pub off_shift_interval: TimeInterval,
    pub toolbox_interval: TimeInterval,
    pub resources: HashSet<Resources>,
}

#[derive(Default)]
pub struct OperationalConfigurationBuilder
{
    availability: BTreeSet<Availability>,
    break_interval: Option<TimeInterval>,
    off_shift_interval: Option<TimeInterval>,
    toolbox_interval: Option<TimeInterval>,
    resources: HashSet<Resources>,
}

impl OperationalConfigurationBuilder
{
    pub fn new() -> Self
    {
        Self::default()
    }

    pub fn add_availability(mut self, availability: Availability) -> Self
    {
        self.availability.insert(availability);
        self
    }

    pub fn availability(mut self, availability: BTreeSet<Availability>) -> Self
    {
        self.availability = availability;
        self
    }

    pub fn add_resource(mut self, resource: Resources) -> Self
    {
        self.resources.insert(resource);
        self
    }

    pub fn resources(mut self, resources: HashSet<Resources>) -> Self
    {
        self.resources = resources;
        self
    }

    pub fn break_interval(mut self, interval: TimeInterval) -> Self
    {
        self.break_interval = Some(interval);
        self
    }

    pub fn off_shift_interval(mut self, interval: TimeInterval) -> Self
    {
        self.off_shift_interval = Some(interval);
        self
    }

    pub fn toolbox_interval(mut self, interval: TimeInterval) -> Self
    {
        self.toolbox_interval = Some(interval);
        self
    }

    pub fn build(self) -> OperationalConfiguration
    {
        OperationalConfiguration {
            availability: self.availability,
            break_interval: self
                .break_interval
                .expect("break_interval should always be set when constructing an Actor"),
            off_shift_interval: self
                .off_shift_interval
                .expect("off_shift_interval should always be set when constructing an Actor"),
            toolbox_interval: self
                .toolbox_interval
                .expect("toolbox_interval should always be set when constructing an Actor"),
            resources: self.resources,
        }
    }
}

impl OperationalConfiguration
{
    pub fn new(
        availability: BTreeSet<Availability>,
        break_interval: TimeInterval,
        off_shift_interval: TimeInterval,
        toolbox_interval: TimeInterval,
        resources: HashSet<Resources>,
    ) -> Self
    {
        Self {
            availability,
            break_interval,
            off_shift_interval,
            toolbox_interval,
            resources,
        }
    }
}

// What should the fields be here?
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct SupervisorConfigurationAll
{
    pub id: ActorCompositeId,
    // FIX
    // This information is found in two different places. That is an
    // error that has to be fixed.
    number_of_supervisor_periods: u64,
}

impl SupervisorConfigurationAll
{
    pub fn new(id: ActorCompositeId, number_of_supervisor_periods: u64) -> Self
    {
        Self {
            id,
            number_of_supervisor_periods,
        }
    }
}

// TODO [ ]
// I do not think that this is relevant any more. It should be deleted and
// not to be seen again.
#[derive(Serialize, Deserialize, Debug)]
pub struct Crew
{
    workers: HashMap<WorkerNumber, Worker>,
}

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub struct WorkerNumber(pub u32);

impl Serialize for WorkerNumber
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

impl<'de> Deserialize<'de> for WorkerNumber
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let worker_number_string = String::deserialize(deserializer).unwrap();
        let worker_number_primitive = worker_number_string.parse::<u32>().unwrap();
        Ok(WorkerNumber(worker_number_primitive))
    }
}

impl Crew
{
    pub fn new(workers: Option<HashMap<WorkerNumber, Worker>>) -> Option<Self>
    {
        workers.map(|workers| Crew { workers })
    }

    pub fn get_workers(&self) -> &HashMap<WorkerNumber, Worker>
    {
        &self.workers
    }
}
