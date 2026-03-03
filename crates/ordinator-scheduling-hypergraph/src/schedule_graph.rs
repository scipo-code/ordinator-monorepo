use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::hash_map::Entry;

use anyhow::Result;
use chrono::Days;
use chrono::Duration;
use chrono::NaiveDate;
use chrono::NaiveTime;
use ordinator_scheduling_environment::SchedulingEnvironment;
use ordinator_scheduling_environment::time_environment::period::Period;
use ordinator_scheduling_environment::work_order::ActivityRelation;
use ordinator_scheduling_environment::work_order::WorkOrder;
use ordinator_scheduling_environment::work_order::WorkOrderNumber;
use ordinator_scheduling_environment::work_order::operation::ActivityNumber;
use ordinator_scheduling_environment::work_order::operation::Work;
use ordinator_scheduling_environment::work_order::operation::operation_info::NumberOfPeople;
use ordinator_scheduling_environment::worker_environment::availability::Availability;
use ordinator_scheduling_environment::worker_environment::resources::Skill;
use ordinator_scheduling_environment::worker_environment::worker::Technician;
use slotmap::SecondaryMap;
use slotmap::SlotMap;
use slotmap::new_key_type;
use tracing::debug;

use crate::derive_instances::ActivityView;
use crate::derive_instances::TechnicianView;
use crate::derive_instances::WeeklyView;
use crate::derive_instances::WeeklyWorkOrderView;

// Type Alias to make reasoning about the indices easier

new_key_type! { pub struct NodeIndex; }
new_key_type! { pub struct EdgeIndex; }

pub type TechnicianId = usize;
pub type StartTime = NaiveTime;
pub type FinishTime = NaiveTime;

#[derive(Clone, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub enum ScheduleGraphErrors
{
    ActivityMissing,
    DayMissing(NaiveDate),
    PeriodDuplicate,
    PeriodMissing,
    SkillMissing,
    WorkOrderActivityMissingSkills,
    WorkOrderDuplicate,
    WorkOrderMissing,
    WorkerUnavailable,
    WorkerMissing,
    WorkerDuplicate,
    ActivityExceedNumberOfPeople,
}

#[derive(Hash, Clone, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub(crate) enum Node
{
    Technician(TechnicianId),
    WorkOrder(WorkOrderNode),
    Activity(ActivityNode),
    Period(Period),
    Skill(Skill),
    Day(NaiveDate),
}

#[derive(Hash, Clone, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub(crate) struct WorkOrderNode
{
    work_order_number: WorkOrderNumber,
    earliest_allowed_starting_date: NaiveDate,
    latest_allowed_finish_date: NaiveDate,
}

#[derive(Hash, Clone, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub(crate) struct ActivityNode
{
    activity_number: ActivityNumber,
    number_of_people: NumberOfPeople,
    work_remaining: Work,
}

// I really do not like this. I think that the idea
// is that you should keep the variants below a fixes
// size no matter what.
// TODO FIX [ ] turn the hyperedges into sum types
#[derive(Clone, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub enum Hyperedge
{
    /// Date specific
    Assign(Option<(StartTime, FinishTime)>, AssignEdge),

    /// FORMAT
    /// `vec![$activity, @technicians, @days]`
    Available(Vec<NodeIndex>),
    Exclude(Vec<NodeIndex>),
    BasicStart(Vec<NodeIndex>),

    WorkOrderToOperations(Vec<NodeIndex>),
    Requires(Vec<NodeIndex>),
    StartStart(Vec<NodeIndex>),
    FinishStart(Vec<NodeIndex>),
    /// Has skill
    HasSkill(Vec<NodeIndex>),
}

impl Hyperedge
{
    pub(crate) fn nodes(&self) -> Vec<NodeIndex>
    {
        match self {
            Hyperedge::Assign(_, assign_edge) => {
                let mut nodes = Vec::new();
                if let Some(activities) = &assign_edge.activity {
                    nodes.extend(activities);
                }
                if let Some(technicians) = &assign_edge.technicians {
                    nodes.extend(technicians);
                }
                if let Some(days) = &assign_edge.days {
                    nodes.extend(days);
                }
                nodes
            }
            Hyperedge::Available(v)
            | Hyperedge::Exclude(v)
            | Hyperedge::BasicStart(v)
            | Hyperedge::WorkOrderToOperations(v)
            | Hyperedge::Requires(v)
            | Hyperedge::StartStart(v)
            | Hyperedge::FinishStart(v)
            | Hyperedge::HasSkill(v) => v.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub struct AssignEdge
{
    // JOB RELATED
    work_order: NodeIndex,
    activity: Option<Vec<NodeIndex>>,
    work_segments: Option<Vec<NodeIndex>>,
    // RESOURCE RELATED
    technicians: Option<Vec<NodeIndex>>,
    // TIME RELATED
    period: NodeIndex,
    days: Option<Vec<NodeIndex>>,
}

#[derive(Debug)]
pub struct SchedulingHypergraph
{
    /// Nodes of the problem
    nodes: SlotMap<NodeIndex, Node>,

    /// Hyperedges to handle all the complex interactions
    hyperedges: SlotMap<EdgeIndex, Hyperedge>,

    /// Adjacency list
    /// To use this you access with a `NodeIndex` and the
    /// then you get a list of hyperedges-given by `EdgeIndex`-that
    /// this node is a part of. These `EdgeIndex`s can then
    /// be used to find the associated `HyperEdge` with
    /// `ScheduleGraph::hyperedges`.
    incidence_list: SecondaryMap<NodeIndex, Vec<EdgeIndex>>,

    /// Indices to look up nodes
    technician_indices: HashMap<TechnicianId, NodeIndex>,
    work_order_indices: HashMap<WorkOrderNumber, NodeIndex>,
    period_indices: HashMap<Period, NodeIndex>,
    skill_indices: HashMap<Skill, NodeIndex>,
    day_indices: BTreeMap<NaiveDate, NodeIndex>,
}

/// Public methods
impl SchedulingHypergraph
{
    pub fn new() -> Self
    {
        Self {
            nodes: SlotMap::with_key(),
            hyperedges: SlotMap::with_key(),
            incidence_list: SecondaryMap::new(),
            technician_indices: HashMap::new(),
            work_order_indices: HashMap::new(),
            period_indices: HashMap::new(),
            skill_indices: HashMap::new(),
            day_indices: BTreeMap::new(),
        }
    }

    /// Build a [`SchedulingHypergraph`] from a [`SchedulingEnvironment`].
    ///
    /// Populates skills, periods, work orders, and technicians from the
    /// environment data.
    pub fn from_scheduling_environment(env: &SchedulingEnvironment) -> Result<Self>
    {
        let mut graph = Self::new();

        // 1. Add all skills referenced by work order activities
        for work_order in env.work_orders.inner.values() {
            for activity in work_order.activities() {
                graph.add_skill(activity.skill());
            }
        }

        // 2. Add all periods from the time environment
        for period in &env.time_environment.periods {
            graph
                .add_period(period.clone())
                .map_err(|e| anyhow::anyhow!("Failed to add period: {e:?}"))?;
        }

        // 3. Add all work orders
        for work_order in env.work_orders.inner.values() {
            graph
                .add_work_order(work_order)
                .map_err(|e| anyhow::anyhow!("Failed to add work order: {e:?}"))?;
        }

        // 4. Add technicians from each asset's actor specifications
        for (_asset, actor_spec) in &env.worker_environment.actor_specification {
            for (index, (id_string, (availabilities, skills))) in
                actor_spec.technician_availability().into_iter().enumerate()
            {
                let _ = id_string;
                let mut builder = Technician::builder(index + 1);
                for skill in &skills {
                    builder = builder.add_skill(*skill);
                }
                let technician = builder.build();

                // Add each availability as a separate technician entry
                // The hypergraph only accepts one availability per add_technician call,
                // and enforces uniqueness, so use the first availability.
                if let Some(availability) = availabilities.into_iter().next() {
                    graph
                        .add_technician(technician, availability)
                        .map_err(|e| anyhow::anyhow!("Failed to add technician: {e:?}"))?;
                }
            }
        }

        Ok(graph)
    }

    pub(crate) fn nodes(&self) -> &SlotMap<NodeIndex, Node>
    {
        &self.nodes
    }

    pub(crate) fn hyperedges(&self) -> &SlotMap<EdgeIndex, Hyperedge>
    {
        &self.hyperedges
    }

    pub(crate) fn incidence_list(&self) -> &SecondaryMap<NodeIndex, Vec<EdgeIndex>>
    {
        &self.incidence_list
    }

    /// Returns the number of nodes in the graph
    pub fn node_count(&self) -> usize
    {
        self.nodes.len()
    }

    /// Returns the number of hyperedges in the graph
    pub fn hyperedge_count(&self) -> usize
    {
        self.hyperedges.len()
    }
}

// impl ScheduleGraph {
//     pub fn work_order_relations(&self, work_order: &WorkOrder) ->
// Result<Vec<()>> }

/// Public API to add [`Nodes`] to the graph.
impl SchedulingHypergraph
{
    pub fn add_skill(&mut self, skill: Skill) -> NodeIndex
    {
        if let Some(&existing) = self.skill_indices.get(&skill) {
            return existing;
        }
        self.add_node(Node::Skill(skill))
    }

    pub fn add_work_order(
        &mut self,
        work_order: &WorkOrder,
    ) -> Result<NodeIndex, ScheduleGraphErrors>
    {
        if !work_order.activities().iter().all(|activity| {
            self.skill_indices
                .keys()
                .any(|&all_skills| all_skills == activity.skill())
        }) {
            return Err(ScheduleGraphErrors::WorkOrderActivityMissingSkills);
        }

        // What should you do with this?
        // TODO [ ] - you need logic here on how to handle assignments.
        //
        // NOTE: It is crucial to understand the difference between the
        // business domain models and the scheduling hypergraph domain
        // model. You have to separate these concepts clearly.
        //
        // NOTE: Start with the simple model
        let day_node_index = self.day_indices.get(&work_order.basic_start()).cloned();
        // .ok_or(ScheduleGraphErrors::DayMissing(work_order.basic_start()))?;

        // Crucial lesson! This cannot come first! You learned something great here!
        let work_order_node_index = match self
            .work_order_indices
            .entry(work_order.work_order_number())
        {
            Entry::Vacant(_new_work_order) => self.add_node(Node::WorkOrder(WorkOrderNode {
                work_order_number: work_order.work_order_number(),
                // TODO: Derive these from WorkOrderPolicies during conversion
                earliest_allowed_starting_date: work_order.basic_start(),
                latest_allowed_finish_date: work_order.basic_start(),
            })),
            Entry::Occupied(_already_inserted_work_order) => {
                return Err(ScheduleGraphErrors::WorkOrderDuplicate);
            }
        };

        // BasicStart node should be optional... Is there even a difference between the
        // basic start and
        let _basic_start_edge_index = day_node_index.map(|day_node_index| {
            self.add_edge(Hyperedge::BasicStart(vec![
                work_order_node_index,
                day_node_index,
            ]))
        });

        let mut previous_activity_node: Option<NodeIndex> = None;
        let activity_relations = work_order.activity_relations();
        for (activity_index, operation) in work_order.activities().iter().enumerate() {
            let activity_node_index = self.add_node(Node::Activity(ActivityNode {
                activity_number: operation.operations_number(),
                number_of_people: operation.number_of_people(),
                work_remaining: *operation.work_remaining(),
            }));
            let skill_node_index = *self
                .skill_indices
                .get(&operation.skill())
                .ok_or(ScheduleGraphErrors::SkillMissing)?;

            self.add_edge(Hyperedge::WorkOrderToOperations(vec![
                work_order_node_index,
                activity_node_index,
            ]));
            self.add_edge(Hyperedge::Requires(vec![
                activity_node_index,
                skill_node_index,
            ]));

            if let Some(prev) = previous_activity_node {
                match activity_relations[activity_index - 1] {
                    ActivityRelation::StartStart => {
                        self.add_edge(Hyperedge::StartStart(vec![prev, activity_node_index]))
                    }
                    ActivityRelation::FinishStart => {
                        self.add_edge(Hyperedge::FinishStart(vec![prev, activity_node_index]))
                    }
                    ActivityRelation::Postpone(_time_delta) => todo!(),
                };
            }
            previous_activity_node = Some(activity_node_index);
        }

        // TODO [x] - add relationships between activities here.

        self.work_order_indices
            .insert(work_order.work_order_number(), work_order_node_index);
        Ok(work_order_node_index)
    }

    pub fn add_period(&mut self, period: Period) -> Result<NodeIndex, ScheduleGraphErrors>
    {
        if self.period_indices.contains_key(&period) {
            return Err(ScheduleGraphErrors::PeriodDuplicate);
        };

        let days_in_period = (0..14)
            .map(|e| *period.start_date() + chrono::Days::new(e))
            .collect::<Vec<_>>();

        for day in days_in_period {
            let day_node = self.add_node(Node::Day(day.date_naive()));
            self.day_indices.insert(day.date_naive(), day_node);
        }

        let node_id = self.add_node(Node::Period(period.clone()));

        self.period_indices.insert(period, node_id);
        Ok(node_id)
    }

    // TODO [ ] - Start here when ready again.
    // Adding a Technician should make an availability to every
    // day that he is available.
    //
    // TODO [ ] - You have to make an edge that has all the `skill`s
    // `days`, `technician`,
    //
    // So adding a `technician` should only create a single node for
    // the technician, all the remaining nodes should always be present.
    //
    // The format is
    //
    // vec![$technician, @skills, @days]
    // I think that you should maybe add a single technician availability at a
    // time instead of what you are doing here. This method is adding n different
    // edges at a time, one for each `availability`. This is of course not the
    // intent of the function. The goal is that the API of the edge methods
    // should only ever create a single edge.
    pub fn add_technician(
        &mut self,
        technician: Technician,
        availability: Availability,
    ) -> Result<EdgeIndex, ScheduleGraphErrors>
    {
        // Check that: worker is not present; skill are present; days are present.
        if self.technician_indices.contains_key(&technician.id()) {
            return Err(ScheduleGraphErrors::WorkerDuplicate);
        }

        let mut skills = vec![];
        for skill in technician.skills() {
            let skill = *self
                .skill_indices
                .get(skill)
                .ok_or(ScheduleGraphErrors::SkillMissing)?;
            skills.push(skill);
        }

        // You have to check and create all the availabilities and then
        // you need to
        //
        // You could wrap this in a SQL database, but this is what is needed to
        // scale correctly.
        let mut single_availability = vec![];

        let length_of_availabilities_in_seconds =
            availability.finish_date() - availability.start_date();
        let number_of_days = length_of_availabilities_in_seconds.num_days();
        for date in (0..=number_of_days).map(|d| availability.start_date() + Duration::days(d)) {
            let day_node = self
                .day_indices
                .get(&date)
                .ok_or(ScheduleGraphErrors::DayMissing(date))?;

            single_availability.push(*day_node);
        }

        let technician_id = self.add_node(Node::Technician(technician.id()));

        //
        let mut edges = vec![technician_id];
        edges.extend(skills);
        edges.extend(single_availability);

        let availability_edge = self.add_edge(Hyperedge::Available(edges));

        Ok(availability_edge)
    }
}

/// Public API to add [`HyperEdges`] to the graph
impl SchedulingHypergraph
{
    // TODO [ ] - this should be formulated as ids... it should be the types that
    // are found inside of the `Nodes` enum variants.
    pub fn add_assignment_work_order(
        &mut self,
        worker: TechnicianId,
        work_order: WorkOrderNumber,
        date: Period,
    ) -> Result<EdgeIndex, ScheduleGraphErrors>
    {
        // This should return an error if the `Nodes` is not present.
        let technicians = *self
            .technician_indices
            .get(&worker)
            .ok_or(ScheduleGraphErrors::WorkerMissing)?;
        let work_order = *self
            .work_order_indices
            .get(&work_order)
            .ok_or(ScheduleGraphErrors::WorkOrderMissing)?;
        let period = *self
            .period_indices
            .get(&date)
            .ok_or(ScheduleGraphErrors::PeriodMissing)?;

        // This is the wrong API for this type of problem.
        // You need to rethink the edges, you should make the
        // code run again and then validate the design in the
        //
        // I think that you should validate with TypeDB. And then
        // move to this. You will lose a lot of time here.
        //
        //
        // START HERE.
        let assign_edge = AssignEdge {
            // TODO [ ] - remove the work order here.
            work_order,
            activity: None,
            work_segments: None,
            technicians: Some(vec![technicians]),
            period,
            days: None,
        };

        let edge = Hyperedge::Assign(None, assign_edge);

        let edge_index = self.hyperedges.insert(edge);
        Ok(edge_index)
    }

    /// Format
    /// vec![$activity, @technicians, @days]
    ///
    /// LIST:
    /// TODO [ ] - Daily hour estimates.
    /// You have to handle partial assignments
    pub fn add_assignment_activity(
        &mut self,
        technicians: Vec<TechnicianId>,
        work_order_number: WorkOrderNumber,
        activity_number: ActivityNumber,
        days: Vec<NaiveDate>,
        start_and_finish_time: (StartTime, FinishTime),
    ) -> Result<EdgeIndex, ScheduleGraphErrors>
    {
        let mut date_node_indices = vec![];
        for naive_date in &days {
            date_node_indices.push(
                *self
                    .day_indices
                    .get(naive_date)
                    .ok_or(ScheduleGraphErrors::DayMissing(*naive_date))?,
            );
        }

        let period = self
            .nodes
            .iter()
            .find_map(|(idx, node)| {
                if let Node::Period(period) = node {
                    if period.contains_date(days[0]) {
                        Some(idx)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .ok_or(ScheduleGraphErrors::PeriodMissing)?;

        let mut technician_node_indices = vec![];
        'technician: for technician_id in &technicians {
            let technician_node_index = self
                .technician_indices
                .get(technician_id)
                .ok_or(ScheduleGraphErrors::WorkerMissing)?;
            technician_node_indices.push(*technician_node_index);

            for availability_hyperedge in
                self.incidence_list[*technician_node_index]
                    .iter()
                    .filter(|&&hyperedge_index| {
                        matches!(self.hyperedges[hyperedge_index], Hyperedge::Available(..))
                    })
            {
                match &self.hyperedges[*availability_hyperedge] {
                    // You have to cover the shift with days. That is the most fundamental here.
                    Hyperedge::Available(availability_nodes) => {
                        let availability_days = availability_nodes
                            .iter()
                            .filter_map(|node_index| match &self.nodes[*node_index] {
                                Node::Day(naive_date) => Some(naive_date),
                                _ => None,
                            })
                            .collect::<Vec<_>>();

                        if days
                            .iter()
                            .all(|activity_day| availability_days.contains(&activity_day))
                        {
                            continue 'technician;
                        };
                    }
                    _ => unreachable!(),
                }
            }
            return Err(ScheduleGraphErrors::WorkerUnavailable);
        }

        // TODO [ ] - Find the availabilities for every technician and make sure that
        // its shift is covered.
        let work_order_node_index = *self
            .work_order_indices
            .get(&work_order_number)
            .ok_or(ScheduleGraphErrors::WorkOrderMissing)?;

        // TODO - [ ] Make a `nodes_in_hyperedge(self, edge_id) -> Vec<Nodes>` method.
        let activity_node_index = *self
            .incidence_list
            .get(work_order_node_index)
            .ok_or(ScheduleGraphErrors::WorkOrderMissing)?
            .iter()
            .find_map(|&hyperedge_index| {
                match &self.hyperedges[hyperedge_index] {
                    // RELEVANT:
                    Hyperedge::WorkOrderToOperations(items) => {
                        items
                            .iter()
                            .find(|&&node_index| match &self.nodes[node_index] {
                                Node::Activity(activity) => {
                                    activity.activity_number == activity_number
                                }
                                _ => false,
                            })
                    }

                    // NOT-RELEVENT:
                    _ => None,
                }
            })
            .ok_or(ScheduleGraphErrors::ActivityMissing)?;

        if let Node::Activity(activity) = &self.nodes[activity_node_index]
            && technicians.len() > activity.number_of_people as usize
        {
            return Err(ScheduleGraphErrors::ActivityExceedNumberOfPeople);
        }

        // Does this actually even need to have the period?
        let assign = AssignEdge {
            work_order: work_order_node_index,
            activity: Some(vec![activity_node_index]),
            work_segments: None,
            technicians: technician_node_indices.into(),
            period,
            days: Some(date_node_indices),
        };

        // TODO [ ] - Add `Day`s as well.
        Ok(self.add_edge(Hyperedge::Assign(Some(start_and_finish_time), assign)))
    }

    // This function should be in a different place in the code. I believe that
    // this is an internal helper function. The user should not be exposed to a
    // `HyperEdge` instance. It should return `Vec<Workers>` or `Vec<WorkOrder>`
    // or `Vec<WorkOrderActivities>`. This should be moved to an Internal API
    // function call.

    /// If the start_naive_date of `EdgeType::Assign(assignment)` in the period
    /// interval the it counts as belonging to that period.
    pub fn find_all_assignments_for_period(
        &self,
        period_start_date: Period,
    ) -> Result<Vec<EdgeIndex>, ScheduleGraphErrors>
    {
        if !self
            .nodes
            .values()
            .any(|e| e == &Node::Period(period_start_date.clone()))
        {
            return Err(ScheduleGraphErrors::PeriodMissing);
        }

        // You cannot make an assignment without a `Period`
        let mut edges = vec![];
        for (edge_index, hyperedge) in self.hyperedges.iter() {
            if let Hyperedge::Assign(_, assign) = hyperedge
                && let Node::Period(period) = &self.nodes[assign.period]
                && period == &period_start_date
            {
                edges.push(edge_index);
            }
        }

        Ok(edges)
    }

    pub fn add_assign_skill_to_worker(
        &mut self,
        worker: TechnicianId,
        skill: Skill,
    ) -> Result<EdgeIndex, ScheduleGraphErrors>
    {
        let worker = self
            .technician_indices
            .get(&worker)
            .ok_or(ScheduleGraphErrors::WorkerMissing)?;
        let skill = self
            .skill_indices
            .get(&skill)
            .ok_or(ScheduleGraphErrors::SkillMissing)?;

        Ok(self.add_edge(Hyperedge::HasSkill(vec![*worker, *skill])))
    }

    /// This method can fail when:
    /// * `WorkOrderNumber` does not exist
    /// * `Period` does not exist.
    /// * The hyperedge between the `WorkOrderNumber` and `Period` already
    ///   exists.
    //[ ] TODO  [ ] exclusion should work on the days as well.
    pub fn add_exclusion(
        &mut self,
        work_order_number: &WorkOrderNumber,
        period: &Period,
    ) -> Result<EdgeIndex, ScheduleGraphErrors>
    {
        let work_order_node_index = self
            .work_order_indices
            .get(work_order_number)
            .ok_or(ScheduleGraphErrors::WorkOrderMissing)?;
        let period_node_index = self
            .period_indices
            .get(period)
            .ok_or(ScheduleGraphErrors::PeriodMissing)?;

        let days_node_indices = self
            .day_indices
            .iter()
            .filter_map(|(&naive_date, &date_index)| {
                if period.start_date().date_naive() <= naive_date
                    && naive_date
                        <= period
                            .start_date()
                            .date_naive()
                            .checked_add_days(Days::new(13))
                            .unwrap()
                {
                    Some(date_index)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let mut final_nodes_in_hyperedge = vec![*work_order_node_index, *period_node_index];
        final_nodes_in_hyperedge.extend(days_node_indices);

        Ok(self.add_edge(Hyperedge::Exclude(final_nodes_in_hyperedge)))
    }
}

/// Extraction methods.
///
/// The primary use cases are:
/// 1. Deriving problem instances for the `Actor`s and their associated
///    `Algorithms`
/// 2. Deriving business information for API end-points in the
///    `ordinator-api-server`
///
/// The ultimate reason for the derivation of the problem instances is
/// simply to create the `Parameters` for the `Algorithm`s.
///
/// DESIGN ESSAY:
/// I am not sure which part of the abstraction should be in the
/// Hypergraph and which should be in the algorithm parameters.
/// The best approach here is probably to make the system work
/// in a way such that the `weekly` word does not enter into the
/// system.
///
/// The `self.hyperedges` and other self fields cannot be accessed outside
/// of the hypergraph.
///
/// One idea is to use the `extract_work_order_view`, `extract_resources_view`,
/// and `extract_time_view` I am not sure that this is the best idea either,
/// to move forward I think that the best decision is to code each of the
/// extractors for each of the schedules and then carefully review the
/// patterns that exists. You cannot forget this task
impl SchedulingHypergraph
{
    pub fn extract_weekly_view(&self) -> WeeklyView
    {
        let mut work_orders = HashMap::new();

        for (&work_order_number, &work_order_node_index) in &self.work_order_indices {
            let mut basic_start_date = None;
            let mut assigned_period = None;
            let mut excluded_periods = HashSet::new();
            let mut activity_node_indices = Vec::new();

            for &edge_idx in &self.incidence_list[work_order_node_index] {
                match &self.hyperedges[edge_idx] {
                    Hyperedge::BasicStart(nodes) => {
                        for &node_index in nodes {
                            if let Node::Day(date) = &self.nodes[node_index] {
                                basic_start_date = Some(*date);
                            }
                        }
                    }
                    Hyperedge::Exclude(nodes) => {
                        for &node_index in nodes {
                            if let Node::Period(period) = &self.nodes[node_index] {
                                excluded_periods.insert(period.clone());
                            }
                        }
                    }
                    Hyperedge::WorkOrderToOperations(nodes) => {
                        for &node_index in nodes {
                            if matches!(&self.nodes[node_index], Node::Activity(_)) {
                                activity_node_indices.push(node_index);
                            }
                        }
                    }
                    _ => {}
                }
            }

            // Assign edges store work_order as a field but don't include it
            // in nodes(), so they are not in the incidence list. Scan all edges.
            for (_, hyperedge) in self.hyperedges.iter() {
                if let Hyperedge::Assign(_, assign_edge) = hyperedge
                    && assign_edge.work_order == work_order_node_index
                    && let Node::Period(period) = &self.nodes[assign_edge.period]
                {
                    assigned_period = Some(period.clone());
                }
            }

            // Build activity views
            let mut activities: Vec<(ActivityNumber, ActivityView)> = Vec::new();
            for &act_ni in &activity_node_indices {
                if let Node::Activity(activity) = &self.nodes[act_ni] {
                    let mut required_skill = None;

                    for &edge_idx in &self.incidence_list[act_ni] {
                        if let Hyperedge::Requires(nodes) = &self.hyperedges[edge_idx] {
                            for &ni in nodes {
                                if let Node::Skill(skill) = &self.nodes[ni] {
                                    required_skill = Some(*skill);
                                }
                            }
                        }
                    }

                    activities.push((
                        activity.activity_number,
                        ActivityView {
                            activity_number: activity.activity_number,
                            number_of_people: activity.number_of_people,
                            work_remaining: activity.work_remaining,
                            required_skill: required_skill
                                .expect("Activity must have a required skill"),
                            relation_to_next: None,
                        },
                    ));
                }
            }

            // Sort by activity_number
            activities.sort_by_key(|(num, _)| *num);

            // Determine relation_to_next from StartStart/FinishStart edges
            for i in 0..activities.len().saturating_sub(1) {
                let current_activity_number = activities[i].0;
                let next_activity_number = activities[i + 1].0;

                // Find the activity node index for the current activity
                let current_node_index = activity_node_indices
                    .iter()
                    .find(|&&ni| {
                        matches!(&self.nodes[ni], Node::Activity(a) if a.activity_number == current_activity_number)
                    })
                    .unwrap();

                for &edge_index in &self.incidence_list[*current_node_index] {
                    match &self.hyperedges[edge_index] {
                        Hyperedge::StartStart(nodes) if nodes.len() == 2 => {
                            if nodes[0] == *current_node_index
                                && let Node::Activity(target) = &self.nodes[nodes[1]]
                                && target.activity_number == next_activity_number
                            {
                                activities[i].1.relation_to_next =
                                    Some(ActivityRelation::StartStart);
                            }
                        }
                        Hyperedge::FinishStart(nodes) if nodes.len() == 2 => {
                            if nodes[0] == *current_node_index
                                && let Node::Activity(target) = &self.nodes[nodes[1]]
                                && target.activity_number == next_activity_number
                            {
                                activities[i].1.relation_to_next =
                                    Some(ActivityRelation::FinishStart);
                            }
                        }
                        _ => {}
                    }
                }
            }

            let activity_views = activities.into_iter().map(|(_, v)| v).collect();
            let latest_allowed_finish_date =
                if let Node::WorkOrder(work_order) = &self.nodes[work_order_node_index] {
                    work_order.latest_allowed_finish_date
                } else {
                    panic!("work_order_nodex_index was not part of the graph")
                };
            work_orders.insert(
                work_order_number,
                WeeklyWorkOrderView {
                    basic_start_date,
                    assigned_period,
                    excluded_periods,
                    activities: activity_views,
                    latest_allowed_finish_date,
                },
            );
        }

        // Collect periods sorted chronologically
        let mut periods: Vec<Period> = self.period_indices.keys().cloned().collect();
        periods.sort();

        // Collect all skills
        let skills: HashSet<Skill> = self.skill_indices.keys().copied().collect();

        // Collect technicians
        let mut technicians = HashMap::new();
        for (&tech_id, &tech_ni) in &self.technician_indices {
            let mut tech_skills = BTreeSet::new();
            let mut available_dates = HashSet::new();

            for &edge_idx in &self.incidence_list[tech_ni] {
                match &self.hyperedges[edge_idx] {
                    Hyperedge::HasSkill(nodes) => {
                        for &ni in nodes {
                            if let Node::Skill(skill) = &self.nodes[ni] {
                                tech_skills.insert(*skill);
                            }
                        }
                    }
                    Hyperedge::Available(nodes) => {
                        for &ni in nodes {
                            if let Node::Day(date) = &self.nodes[ni] {
                                available_dates.insert(*date);
                            }
                        }
                    }
                    _ => {}
                }
            }

            technicians.insert(
                tech_id,
                TechnicianView {
                    skills: tech_skills,
                    available_dates,
                },
            );
        }

        WeeklyView {
            work_orders,
            periods,
            skills,
            technicians,
        }
    }
}

/// Private methods.
///
/// [`NodeIndex`] and [`EdgeIndex`] are not allowed to be a part of the
/// public API of the type. The graph should only expose domain types
/// found in `ordinator-scheduling-environment`
impl SchedulingHypergraph
{
    fn add_node(&mut self, node: Node) -> NodeIndex
    {
        let node_index = self.nodes.insert(node);
        let node_ref = self.nodes[node_index].clone();
        let none_checker = match node_ref {
            Node::Technician(worker) => self.technician_indices.insert(worker, node_index),
            Node::WorkOrder(work_order) => {
                self.work_order_indices
                    .insert(work_order.work_order_number, node_index)
            }
            Node::Period(naive_date) => self.period_indices.insert(naive_date, node_index),
            Node::Skill(skills) => self.skill_indices.insert(skills, node_index),
            Node::Activity(a) => {
                debug!(target: "developer", activity = ?a, "No node index for `Activities`");
                None
            }
            Node::Day(naive_date) => self.day_indices.insert(naive_date, node_index),
        };
        assert!(none_checker.is_none());

        self.incidence_list.insert(node_index, vec![]);

        node_index
    }

    fn add_edge(&mut self, edge: Hyperedge) -> EdgeIndex
    {
        let node_indices = edge.nodes();
        let edge_index = self.hyperedges.insert(edge);

        for node_index in node_indices {
            self.incidence_list[node_index].push(edge_index);
        }
        edge_index
    }
}
impl Default for SchedulingHypergraph
{
    fn default() -> Self
    {
        Self::new()
    }
}

#[cfg(test)]
mod tests
{
    use std::collections::HashSet;

    use chrono::Duration;
    use chrono::NaiveDate;
    use chrono::NaiveTime;
    use ordinator_scheduling_environment::work_order::ActivityRelation;
    use ordinator_scheduling_environment::work_order::WorkOrder;
    use ordinator_scheduling_environment::work_order::WorkOrderNumber;
    use ordinator_scheduling_environment::work_order::operation::Operation;
    use ordinator_scheduling_environment::work_order::operation::Work;
    use ordinator_scheduling_environment::worker_environment::availability::Availability;
    use ordinator_scheduling_environment::worker_environment::resources::Skill;
    use ordinator_scheduling_environment::worker_environment::worker::Technician;

    use super::Node;
    use super::SchedulingHypergraph;
    use super::WorkOrderNode;
    use crate::schedule_graph::Hyperedge;
    use crate::schedule_graph::Period;
    use crate::schedule_graph::ScheduleGraphErrors;

    #[test]
    fn test_schedule_graph_new()
    {
        let mut schedule_graph = SchedulingHypergraph::new();

        let date = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        let index_worker = schedule_graph.add_node(Node::Technician(1234));
        let index_workorder = schedule_graph.add_node(Node::WorkOrder(WorkOrderNode {
            work_order_number: WorkOrderNumber(1122334455),
            earliest_allowed_starting_date: date,
            latest_allowed_finish_date: date,
        }));
        let index_period = schedule_graph
            .add_period(Period::from_start_date(date))
            .unwrap();

        assert!(schedule_graph.nodes[index_worker] == Node::Technician(1234));
        assert!(
            schedule_graph.nodes[index_workorder]
                == Node::WorkOrder(WorkOrderNode {
                    work_order_number: WorkOrderNumber(1122334455),
                    earliest_allowed_starting_date: date,
                    latest_allowed_finish_date: date,
                })
        );
        assert!(schedule_graph.nodes[index_period] == Node::Period(Period::from_start_date(date)));

        schedule_graph
            .add_assignment_work_order(
                1234,
                WorkOrderNumber(1122334455),
                Period::from_start_date(date),
            )
            .unwrap();
    }

    #[test]
    fn test_add_work_order()
    {
        let mut schedule_graph = SchedulingHypergraph::new();

        let _skill_node_id = schedule_graph.add_node(Node::Skill(Skill::MtnMech));

        let basic_start_date = NaiveDate::from_ymd_opt(2025, 1, 13).unwrap();
        let work_order = WorkOrder::new(
            1122334455,
            basic_start_date,
            vec![
                Operation::new(10, 1, Skill::MtnMech),
                Operation::new(20, 1, Skill::MtnMech),
                Operation::new(30, 1, Skill::MtnMech),
            ],
        )
        .unwrap();

        assert_eq!(
            schedule_graph.add_work_order(&work_order),
            Err(ScheduleGraphErrors::DayMissing(basic_start_date))
        );

        let _period_node_id = schedule_graph
            .add_period(Period::from_start_date(basic_start_date))
            .unwrap();
        let work_order_node_id = schedule_graph
            .add_work_order(&work_order)
            .expect("Could not add work order");

        assert_eq!(
            schedule_graph.nodes[work_order_node_id],
            Node::WorkOrder(WorkOrderNode {
                work_order_number: WorkOrderNumber(1122334455),
                earliest_allowed_starting_date: basic_start_date,
                latest_allowed_finish_date: basic_start_date,
            })
        );

        // Collect activity node indices through the graph structure
        let mut activity_node_indices: Vec<super::NodeIndex> = vec![];
        for &edge_idx in &schedule_graph.incidence_list[work_order_node_id] {
            if let Hyperedge::WorkOrderToOperations(nodes) = &schedule_graph.hyperedges[edge_idx] {
                for &n in nodes {
                    if matches!(&schedule_graph.nodes[n], Node::Activity(_)) {
                        activity_node_indices.push(n);
                    }
                }
            }
        }

        assert_eq!(activity_node_indices.len(), 3);
        assert_eq!(
            schedule_graph.nodes[activity_node_indices[0]],
            Node::Activity(crate::schedule_graph::ActivityNode {
                activity_number: 10,
                number_of_people: 1,
                work_remaining: Work::default(),
            })
        );
        assert_eq!(
            schedule_graph.nodes[activity_node_indices[1]],
            Node::Activity(crate::schedule_graph::ActivityNode {
                activity_number: 20,
                number_of_people: 1,
                work_remaining: Work::default(),
            })
        );
        assert_eq!(
            schedule_graph.nodes[activity_node_indices[2]],
            Node::Activity(crate::schedule_graph::ActivityNode {
                activity_number: 30,
                number_of_people: 1,
                work_remaining: Work::default(),
            })
        );

        // Verify FinishStart edge between activity 0 and 1
        assert!(
            schedule_graph.incidence_list[activity_node_indices[0]]
                .iter()
                .any(|&e| {
                    schedule_graph.hyperedges[e]
                        == Hyperedge::FinishStart(vec![
                            activity_node_indices[0],
                            activity_node_indices[1],
                        ])
                })
        );
        // Verify FinishStart edge between activity 1 and 2
        assert!(
            schedule_graph.incidence_list[activity_node_indices[1]]
                .iter()
                .any(|&e| {
                    schedule_graph.hyperedges[e]
                        == Hyperedge::FinishStart(vec![
                            activity_node_indices[1],
                            activity_node_indices[2],
                        ])
                })
        );
        // Verify NO FinishStart edge where the last activity is the source (first
        // element)
        assert!(
            !schedule_graph.incidence_list[activity_node_indices[2]]
                .iter()
                .any(|&e| {
                    matches!(&schedule_graph.hyperedges[e],
                        Hyperedge::FinishStart(nodes) if nodes[0] == activity_node_indices[2])
                })
        );

        let basic_start_day_node_id = *schedule_graph.day_indices.get(&basic_start_date).unwrap();

        let work_order_edge_ids = &schedule_graph.incidence_list[work_order_node_id];

        for &edge_id in work_order_edge_ids {
            let hyper_edge = &schedule_graph.hyperedges[edge_id];
            match hyper_edge {
                Hyperedge::Assign(..) => todo!(),
                Hyperedge::Available(_) => todo!(),
                Hyperedge::BasicStart(nodes) => {
                    assert_eq!(basic_start_day_node_id, nodes[1]);
                    assert_eq!(work_order_node_id, nodes[0]);
                }
                Hyperedge::WorkOrderToOperations(nodes) => {
                    assert_eq!(work_order_node_id, nodes[0]);
                }
                Hyperedge::Requires(_) => todo!(),
                Hyperedge::StartStart(_) => todo!(),
                Hyperedge::FinishStart(_) => todo!(),
                Hyperedge::Exclude(_) => todo!(),
                Hyperedge::HasSkill(_) => todo!(),
            }
        }
    }

    #[test]
    fn test_add_technician()
    {
        let mut schedule_graph = SchedulingHypergraph::new();

        let start = NaiveDate::from_ymd_opt(2025, 1, 1)
            .unwrap()
            .and_hms_opt(8, 0, 0)
            .unwrap();
        let end = NaiveDate::from_ymd_opt(2025, 1, 7)
            .unwrap()
            .and_hms_opt(17, 0, 0)
            .unwrap();

        let technician = Technician::builder(1)
            .add_availability(start, end)
            .unwrap()
            .add_skill(Skill::MtnMech)
            .build();

        let skill_node = schedule_graph.add_node(Node::Skill(Skill::MtnMech));

        schedule_graph
            .add_period(Period::from_start_date(start.date()))
            .unwrap();

        let availability = Availability::from_naive(start, end);

        let availability_edge_idx = schedule_graph
            .add_technician(technician, availability)
            .unwrap();

        // Verify skill node
        assert_eq!(
            schedule_graph.nodes[skill_node],
            Node::Skill(Skill::MtnMech)
        );

        // Verify day nodes via day_indices
        for day_offset in 0..14 {
            let date = start.date() + Duration::days(day_offset);
            let day_node_idx = *schedule_graph.day_indices.get(&date).unwrap();
            assert_eq!(schedule_graph.nodes[day_node_idx], Node::Day(date));
        }

        // Verify period node
        let period_node_idx = *schedule_graph
            .period_indices
            .get(&Period::from_start_date(start.date()))
            .unwrap();
        assert_eq!(
            schedule_graph.nodes[period_node_idx],
            Node::Period(Period::from_start_date(start.date()))
        );

        // Verify the availability edge contains the right nodes
        let edge_nodes = schedule_graph.hyperedges[availability_edge_idx].nodes();
        let tech_node = *schedule_graph.technician_indices.get(&1).unwrap();
        assert!(edge_nodes.contains(&tech_node));
        assert!(edge_nodes.contains(&skill_node));
        // 1 technician + 1 skill + 7 days = 9
        assert_eq!(edge_nodes.len(), 9);

        // Verify incidence list entries
        assert!(schedule_graph.incidence_list[tech_node].contains(&availability_edge_idx));
        assert!(schedule_graph.incidence_list[skill_node].contains(&availability_edge_idx));
        for day_offset in 0..7 {
            let date = start.date() + Duration::days(day_offset);
            let day_node_idx = *schedule_graph.day_indices.get(&date).unwrap();
            assert!(schedule_graph.incidence_list[day_node_idx].contains(&availability_edge_idx));
        }
    }

    #[test]
    fn test_neighbors()
    {
        let mut schedule_graph = SchedulingHypergraph::new();

        let date = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        let technician_node_1 = Node::Technician(1234);
        let technician_node_index_1 = schedule_graph.add_node(technician_node_1.clone());
        let work_order_node_1 = Node::WorkOrder(WorkOrderNode {
            work_order_number: WorkOrderNumber(1122334455),
            earliest_allowed_starting_date: date,
            latest_allowed_finish_date: date,
        });
        let work_order_node_index_1 = schedule_graph.add_node(work_order_node_1.clone());
        let period_node_1 = Node::Period(Period::from_start_date(date));
        let period_node_index_1 = schedule_graph.add_node(period_node_1.clone());

        assert!(schedule_graph.nodes[technician_node_index_1] == technician_node_1);
        assert!(schedule_graph.nodes[work_order_node_index_1] == work_order_node_1);
        assert!(schedule_graph.nodes[period_node_index_1] == period_node_1);

        // Using builder to make complex edges will become crucial for the
        // system to function correctly.
        let assignment_edge_index_0 = schedule_graph
            .add_assignment_work_order(
                1234,
                WorkOrderNumber(1122334455),
                Period::from_start_date(date),
            )
            .unwrap();

        let technician_node_2 = Node::Technician(1236);
        let technician_node_index_2 = schedule_graph.add_node(technician_node_2.clone());
        let work_order_node_2 = Node::WorkOrder(WorkOrderNode {
            work_order_number: WorkOrderNumber(1122334456),
            earliest_allowed_starting_date: date,
            latest_allowed_finish_date: date,
        });
        let work_order_node_index_2 = schedule_graph.add_node(work_order_node_2.clone());

        assert!(schedule_graph.nodes[technician_node_index_2] == technician_node_2);
        assert!(schedule_graph.nodes[work_order_node_index_2] == work_order_node_2);
        assert!(schedule_graph.nodes[period_node_index_1] == period_node_1);
        let assignment_edge_index_1 = schedule_graph
            .add_assignment_work_order(
                1236,
                WorkOrderNumber(1122334456),
                Period::from_start_date(date),
            )
            .unwrap();

        let assignment_edges = schedule_graph
            .find_all_assignments_for_period(Period::from_start_date(date))
            .unwrap();

        assert_eq!(assignment_edges[0], assignment_edge_index_0);

        assert_eq!(assignment_edges[1], assignment_edge_index_1);
    }

    #[test]
    fn test_skill_assign()
    {
        let mut schedule_graph = SchedulingHypergraph::new();

        let _worker_node = schedule_graph.add_node(Node::Technician(1234));
        let _skill_node = schedule_graph.add_node(Node::Skill(Skill::MtnMech));

        assert!(
            schedule_graph
                .add_assign_skill_to_worker(1234, Skill::MtnMech)
                .is_ok()
        );
        assert_eq!(
            schedule_graph.add_assign_skill_to_worker(1234, Skill::MtnElec),
            Err(ScheduleGraphErrors::SkillMissing)
        );
    }

    #[test]
    fn test_add_period()
    {
        let mut schedule_state = SchedulingHypergraph::new();

        let period_1 = Period::from_start_date(NaiveDate::from_ymd_opt(2025, 1, 13).unwrap());
        let period_2 = Period::from_start_date(NaiveDate::from_ymd_opt(2025, 1, 27).unwrap());
        let period_3 = Period::from_start_date(NaiveDate::from_ymd_opt(2025, 2, 10).unwrap());

        let _node_id = schedule_state.add_period(period_1.clone()).unwrap();
        let _node_id = schedule_state.add_period(period_2.clone()).unwrap();
        let _node_id = schedule_state.add_period(period_3.clone()).unwrap();

        let node_id = schedule_state.add_period(period_3.clone());

        assert!(schedule_state.period_indices.contains_key(&period_1));
        assert!(schedule_state.period_indices.contains_key(&period_2));
        assert!(schedule_state.period_indices.contains_key(&period_3));

        assert!(node_id == Err(ScheduleGraphErrors::PeriodDuplicate));
        let start_date = NaiveDate::from_ymd_opt(2025, 1, 13).unwrap();
        let finish_date = NaiveDate::from_ymd_opt(2025, 2, 23).unwrap();

        let mut date = start_date;
        while date <= finish_date {
            assert!(
                schedule_state.day_indices.contains_key(&date),
                "Missing date: {date}"
            );
            date += Duration::days(1);
        }

        let hash_set_days = schedule_state
            .nodes
            .values()
            .filter(|e| matches!(e, Node::Day(_)))
            .collect::<HashSet<_>>();

        let vec_days = schedule_state
            .nodes
            .values()
            .filter(|e| matches!(e, Node::Day(_)))
            .collect::<Vec<_>>();

        assert_eq!(hash_set_days.len(), vec_days.len())
    }

    #[test]
    fn test_multi_directional_hypergraph()
    {
        let mut schedule_graph = SchedulingHypergraph::new();

        let d = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        let node_0 = Node::WorkOrder(WorkOrderNode { work_order_number: WorkOrderNumber(1111990000), earliest_allowed_starting_date: d, latest_allowed_finish_date: d });
        let node_1 = Node::WorkOrder(WorkOrderNode { work_order_number: WorkOrderNumber(1111990001), earliest_allowed_starting_date: d, latest_allowed_finish_date: d });
        let node_2 = Node::WorkOrder(WorkOrderNode { work_order_number: WorkOrderNumber(1111990002), earliest_allowed_starting_date: d, latest_allowed_finish_date: d });
        let node_3 = Node::WorkOrder(WorkOrderNode { work_order_number: WorkOrderNumber(1111990003), earliest_allowed_starting_date: d, latest_allowed_finish_date: d });
        let node_4 = Node::WorkOrder(WorkOrderNode { work_order_number: WorkOrderNumber(1111990004), earliest_allowed_starting_date: d, latest_allowed_finish_date: d });
        let node_5 = Node::WorkOrder(WorkOrderNode { work_order_number: WorkOrderNumber(1111990005), earliest_allowed_starting_date: d, latest_allowed_finish_date: d });
        let node_6 = Node::WorkOrder(WorkOrderNode { work_order_number: WorkOrderNumber(1111990006), earliest_allowed_starting_date: d, latest_allowed_finish_date: d });
        let node_7 = Node::WorkOrder(WorkOrderNode { work_order_number: WorkOrderNumber(1111990007), earliest_allowed_starting_date: d, latest_allowed_finish_date: d });

        let node_index_0 = schedule_graph.add_node(node_0);
        let node_index_1 = schedule_graph.add_node(node_1);
        let node_index_2 = schedule_graph.add_node(node_2);
        let node_index_3 = schedule_graph.add_node(node_3);
        let node_index_4 = schedule_graph.add_node(node_4);
        let node_index_5 = schedule_graph.add_node(node_5);
        let node_index_6 = schedule_graph.add_node(node_6);
        let node_index_7 = schedule_graph.add_node(node_7);

        let edge_index_0 = schedule_graph.add_edge(Hyperedge::Available(vec![
            node_index_0,
            node_index_2,
            node_index_4,
            node_index_6,
        ]));
        let edge_index_1 = schedule_graph.add_edge(Hyperedge::Available(vec![
            node_index_1,
            node_index_3,
            node_index_5,
            node_index_7,
        ]));
        let edge_index_2 = schedule_graph.add_edge(Hyperedge::Available(vec![
            node_index_0,
            node_index_3,
            node_index_6,
        ]));

        assert_eq!(
            schedule_graph.incidence_list[node_index_0],
            vec![edge_index_0, edge_index_2]
        );
        assert_eq!(
            schedule_graph.incidence_list[node_index_1],
            vec![edge_index_1]
        );
        assert_eq!(
            schedule_graph.incidence_list[node_index_2],
            vec![edge_index_0]
        );
        assert_eq!(
            schedule_graph.incidence_list[node_index_3],
            vec![edge_index_1, edge_index_2]
        );
        assert_eq!(
            schedule_graph.incidence_list[node_index_4],
            vec![edge_index_0]
        );
        assert_eq!(
            schedule_graph.incidence_list[node_index_5],
            vec![edge_index_1]
        );
        assert_eq!(
            schedule_graph.incidence_list[node_index_6],
            vec![edge_index_0, edge_index_2]
        );
        assert_eq!(
            schedule_graph.incidence_list[node_index_7],
            vec![edge_index_1]
        );
    }

    #[test]
    fn test_add_exclusion()
    {
        let mut schedule_graph = SchedulingHypergraph::new();

        let basic_start_date = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        let work_order = WorkOrder::new(1111990000, basic_start_date, vec![]).unwrap();

        let period = Period::from_start_date(basic_start_date);

        let period_node_index = schedule_graph.add_period(period.clone()).unwrap();
        let work_order_node_index = schedule_graph.add_work_order(&work_order).unwrap();

        let exclusion_edge_index = schedule_graph
            .add_exclusion(&WorkOrderNumber(1111990000), &period)
            .unwrap();

        // Verify the exclusion edge contains the right nodes
        let exclusion_edge = &schedule_graph.hyperedges[exclusion_edge_index];
        if let Hyperedge::Exclude(nodes) = exclusion_edge {
            assert!(nodes.contains(&work_order_node_index));
            assert!(nodes.contains(&period_node_index));
            // All 14 day nodes should be in the exclusion
            for day_offset in 0..14 {
                let date = basic_start_date + Duration::days(day_offset);
                let day_idx = *schedule_graph.day_indices.get(&date).unwrap();
                assert!(nodes.contains(&day_idx));
            }
            // 1 work_order + 1 period + 14 days
            assert_eq!(nodes.len(), 16);
        } else {
            panic!("Expected Exclude hyperedge");
        }

        assert!(
            schedule_graph.incidence_list[work_order_node_index].contains(&exclusion_edge_index)
        );
        assert!(schedule_graph.incidence_list[period_node_index].contains(&exclusion_edge_index));
    }

    #[test]
    fn test_add_assignment_activity()
    {
        let mut schedule_graph = SchedulingHypergraph::new();

        // Create test dates
        let basic_start_date_0 = NaiveDate::from_ymd_opt(2025, 1, 13).unwrap();
        let basic_start_date_1 = NaiveDate::from_ymd_opt(2025, 1, 27).unwrap();
        let availability_start_0 = basic_start_date_0.and_hms_opt(8, 0, 0).unwrap();
        let availability_end_0 = basic_start_date_0.and_hms_opt(17, 0, 0).unwrap();
        let availability_start_1 = basic_start_date_1.and_hms_opt(8, 0, 0).unwrap();
        let availability_end_1 = basic_start_date_1.and_hms_opt(17, 0, 0).unwrap();

        // Add required skills first
        let _skill_node_mech = schedule_graph.add_node(Node::Skill(Skill::MtnMech));
        let _skill_node_elec = schedule_graph.add_node(Node::Skill(Skill::MtnElec));

        // Add period (creates day nodes)
        let period = Period::from_start_date(basic_start_date_0);
        let _period_node_index_0 = schedule_graph.add_period(period).unwrap();
        let period_1 = Period::from_start_date(basic_start_date_1);
        let _period_node_index_1 = schedule_graph.add_period(period_1).unwrap();

        // Create WorkOrder with activities
        let work_order = WorkOrder::new(
            1122334455,
            basic_start_date_0,
            vec![
                Operation::new(10, 2, Skill::MtnMech), // Activity 10, 2 hours, MtnMech skill
                Operation::new(20, 3, Skill::MtnElec), // Activity 20, 3 hours, MtnElec skill
            ],
        )
        .unwrap();

        // Add WorkOrder to graph
        let _work_order_node_id = schedule_graph.add_work_order(&work_order).unwrap();

        // Create 2 Technicians using builder pattern
        let technician_1 = Technician::builder(1001)
            .add_availability(availability_start_0, availability_end_0)
            .unwrap()
            .add_skill(Skill::MtnMech)
            .build();

        let technician_2 = Technician::builder(1002)
            .add_availability(availability_start_1, availability_end_1)
            .unwrap()
            .add_skill(Skill::MtnElec)
            .build();

        let technician_3 = Technician::builder(1003)
            .add_availability(availability_start_0, availability_end_0)
            .unwrap()
            .add_skill(Skill::MtnElec)
            .build();

        // Add technicians to graph
        let availability_1 = Availability::from_naive(availability_start_0, availability_end_0);
        let availability_2 = Availability::from_naive(availability_start_1, availability_end_1);
        let availability_3 = Availability::from_naive(availability_start_0, availability_end_0);

        let _tech_edge_1 = schedule_graph
            .add_technician(technician_1, availability_1)
            .unwrap();
        let _tech_edge_2 = schedule_graph
            .add_technician(technician_2, availability_2)
            .unwrap();
        let _tech_edge_3 = schedule_graph
            .add_technician(technician_3, availability_3)
            .unwrap();

        // Test add_assignment_activity with multiple technicians
        let assignment_edge_error = schedule_graph.add_assignment_activity(
            vec![1001, 1002],            // technician_ids
            WorkOrderNumber(1122334455), // work_order_number
            10,                          // activity_number
            vec![basic_start_date_0],    // days
            (
                NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
                NaiveTime::from_hms_opt(11, 0, 0).unwrap(),
            ), // start and finish time
        );

        assert_eq!(
            assignment_edge_error,
            Err(ScheduleGraphErrors::WorkerUnavailable)
        );

        let assignment_edge = schedule_graph
            .add_assignment_activity(
                vec![1001, 1003],            // technician_ids
                WorkOrderNumber(1122334455), // work_order_number
                10,                          // activity_number
                vec![basic_start_date_0],    // days
                (
                    NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
                    NaiveTime::from_hms_opt(11, 0, 0).unwrap(),
                ), // start and finish time
            )
            .unwrap();
        // Should you include the

        // Verify the assignment was created
        let hyperedge = &schedule_graph.hyperedges[assignment_edge];

        // Should be an assignment edge
        assert!(matches!(hyperedge, Hyperedge::Assign(Some(_), _)));

        // Should contain activity + 2 technicians + 1 day = 4 nodes
        assert_eq!(hyperedge.nodes().len(), 4); // activity + 2 technicians + 1 day

        // Verify both technician nodes are in the assignment
        let technician_1_node_id = *schedule_graph.technician_indices.get(&1001).unwrap();
        let technician_3_node_id = *schedule_graph.technician_indices.get(&1003).unwrap();
        assert!(hyperedge.nodes().contains(&technician_1_node_id));
        assert!(hyperedge.nodes().contains(&technician_3_node_id));

        // Verify the assignment shows up in both technicians' incidence lists
        assert!(schedule_graph.incidence_list[technician_1_node_id].contains(&assignment_edge));
        assert!(schedule_graph.incidence_list[technician_3_node_id].contains(&assignment_edge));

        // Verify the activity node is in the assignment
        let day_node_id = *schedule_graph.day_indices.get(&basic_start_date_0).unwrap();
        assert!(hyperedge.nodes().contains(&day_node_id));
    }

    #[test]
    fn test_extract_weekly_view_basic()
    {
        let mut graph = SchedulingHypergraph::new();

        let basic_start = NaiveDate::from_ymd_opt(2025, 1, 13).unwrap();
        let period = Period::from_start_date(basic_start);

        graph.add_skill(Skill::MtnMech);
        graph.add_skill(Skill::MtnElec);
        graph.add_period(period.clone()).unwrap();

        let work_order = WorkOrder::new(
            100,
            basic_start,
            vec![
                Operation::new(10, 1, Skill::MtnMech),
                Operation::new(20, 2, Skill::MtnElec),
            ],
        )
        .unwrap();
        graph.add_work_order(&work_order).unwrap();

        let avail_start = basic_start.and_hms_opt(8, 0, 0).unwrap();
        let avail_end = basic_start.and_hms_opt(17, 0, 0).unwrap();
        let technician = Technician::builder(1)
            .add_availability(avail_start, avail_end)
            .unwrap()
            .add_skill(Skill::MtnMech)
            .build();
        graph
            .add_technician(technician, Availability::from_naive(avail_start, avail_end))
            .unwrap();
        graph.add_assign_skill_to_worker(1, Skill::MtnMech).unwrap();

        let view = graph.extract_weekly_view();

        // Periods
        assert_eq!(view.periods.len(), 1);
        assert_eq!(view.periods[0], period);

        // Skills
        assert_eq!(view.skills.len(), 2);
        assert!(view.skills.contains(&Skill::MtnMech));
        assert!(view.skills.contains(&Skill::MtnElec));

        // Work orders
        assert_eq!(view.work_orders.len(), 1);
        let wo_view = &view.work_orders[&WorkOrderNumber(100)];
        assert_eq!(wo_view.basic_start_date, Some(basic_start));
        assert!(wo_view.assigned_period.is_none());
        assert!(wo_view.excluded_periods.is_empty());

        // Activities sorted by number
        assert_eq!(wo_view.activities.len(), 2);
        assert_eq!(wo_view.activities[0].activity_number, 10);
        assert_eq!(wo_view.activities[0].number_of_people, 1);
        assert_eq!(wo_view.activities[0].required_skill, Skill::MtnMech);
        assert_eq!(wo_view.activities[0].work_remaining, Work::default());
        assert_eq!(wo_view.activities[1].activity_number, 20);
        assert_eq!(wo_view.activities[1].number_of_people, 2);
        assert_eq!(wo_view.activities[1].required_skill, Skill::MtnElec);

        // Technician
        assert_eq!(view.technicians.len(), 1);
        let tech_view = &view.technicians[&1];
        assert!(tech_view.skills.contains(&Skill::MtnMech));
        assert!(!tech_view.available_dates.is_empty());
    }

    #[test]
    fn test_extract_weekly_view_exclusions()
    {
        let mut graph = SchedulingHypergraph::new();

        let start_1 = NaiveDate::from_ymd_opt(2025, 1, 13).unwrap();
        let start_2 = NaiveDate::from_ymd_opt(2025, 1, 27).unwrap();
        let period_1 = Period::from_start_date(start_1);
        let period_2 = Period::from_start_date(start_2);

        graph.add_skill(Skill::MtnMech);
        graph.add_period(period_1.clone()).unwrap();
        graph.add_period(period_2.clone()).unwrap();

        let work_order =
            WorkOrder::new(200, start_1, vec![Operation::new(10, 1, Skill::MtnMech)]).unwrap();
        graph.add_work_order(&work_order).unwrap();

        graph
            .add_exclusion(&WorkOrderNumber(200), &period_1)
            .unwrap();
        graph
            .add_exclusion(&WorkOrderNumber(200), &period_2)
            .unwrap();

        let view = graph.extract_weekly_view();
        let wo_view = &view.work_orders[&WorkOrderNumber(200)];

        assert_eq!(wo_view.excluded_periods.len(), 2);
        assert!(wo_view.excluded_periods.contains(&period_1));
        assert!(wo_view.excluded_periods.contains(&period_2));
    }

    #[test]
    fn test_extract_weekly_view_assignments()
    {
        let mut graph = SchedulingHypergraph::new();

        let start = NaiveDate::from_ymd_opt(2025, 1, 13).unwrap();
        let period = Period::from_start_date(start);

        graph.add_skill(Skill::MtnMech);
        graph.add_period(period.clone()).unwrap();

        let work_order =
            WorkOrder::new(300, start, vec![Operation::new(10, 1, Skill::MtnMech)]).unwrap();
        graph.add_work_order(&work_order).unwrap();

        let avail_start = start.and_hms_opt(8, 0, 0).unwrap();
        let avail_end = start.and_hms_opt(17, 0, 0).unwrap();
        let technician = Technician::builder(1)
            .add_availability(avail_start, avail_end)
            .unwrap()
            .add_skill(Skill::MtnMech)
            .build();
        graph
            .add_technician(technician, Availability::from_naive(avail_start, avail_end))
            .unwrap();

        graph
            .add_assignment_work_order(1, WorkOrderNumber(300), period.clone())
            .unwrap();

        let view = graph.extract_weekly_view();
        let wo_view = &view.work_orders[&WorkOrderNumber(300)];

        assert_eq!(wo_view.assigned_period, Some(period));
    }

    #[test]
    fn test_extract_weekly_view_activity_relations()
    {
        let mut graph = SchedulingHypergraph::new();

        let start = NaiveDate::from_ymd_opt(2025, 1, 13).unwrap();
        let period = Period::from_start_date(start);

        graph.add_skill(Skill::MtnMech);
        graph.add_period(period.clone()).unwrap();

        // WorkOrder::new creates FinishStart relations between consecutive activities
        let work_order = WorkOrder::new(
            400,
            start,
            vec![
                Operation::new(10, 1, Skill::MtnMech),
                Operation::new(20, 1, Skill::MtnMech),
                Operation::new(30, 1, Skill::MtnMech),
            ],
        )
        .unwrap();
        graph.add_work_order(&work_order).unwrap();

        let view = graph.extract_weekly_view();
        let wo_view = &view.work_orders[&WorkOrderNumber(400)];

        assert_eq!(wo_view.activities.len(), 3);
        assert_eq!(wo_view.activities[0].activity_number, 10);
        assert_eq!(wo_view.activities[1].activity_number, 20);
        assert_eq!(wo_view.activities[2].activity_number, 30);

        // First two activities should have FinishStart relation_to_next
        assert!(
            matches!(
                wo_view.activities[0].relation_to_next,
                Some(ActivityRelation::FinishStart)
            ),
            "Expected FinishStart for activity 10, got {:?}",
            wo_view.activities[0].relation_to_next
        );
        assert!(
            matches!(
                wo_view.activities[1].relation_to_next,
                Some(ActivityRelation::FinishStart)
            ),
            "Expected FinishStart for activity 20, got {:?}",
            wo_view.activities[1].relation_to_next
        );
        // Last activity should have no relation_to_next
        assert!(wo_view.activities[2].relation_to_next.is_none());
    }
}
