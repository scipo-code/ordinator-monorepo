use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::hash_map::Entry;

use chrono::Days;
use chrono::Duration;
use chrono::NaiveDate;
use chrono::NaiveTime;
use ordinator_scheduling_environment::time_environment::period::Period;
use ordinator_scheduling_environment::work_order::ActivityRelation;
use ordinator_scheduling_environment::work_order::WorkOrder;
use ordinator_scheduling_environment::work_order::WorkOrderNumber;
use ordinator_scheduling_environment::work_order::operation::ActivityNumber;
use ordinator_scheduling_environment::work_order::operation::operation_info::NumberOfPeople;
use ordinator_scheduling_environment::worker_environment::availability::Availability;
use ordinator_scheduling_environment::worker_environment::resources::Skill;
use ordinator_scheduling_environment::worker_environment::worker::Technician;
use tracing::debug;

// Type Alias to make reasoning about the indices easier
pub type NodeIndex = usize;
pub type EdgeIndex = usize;
pub type TechnicianId = usize;
pub type StartTime = NaiveTime;
pub type FinishTime = NaiveTime;

const HYPEREDGE_NODE_SEPERATOR: usize = usize::MAX; // Reserved sentinel value

#[derive(Clone, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub enum ScheduleGraphErrors
{
    ActivityMissing,
    DayMissing,
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

#[derive(Clone, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub(crate) struct HyperEdge
{
    edge_type: EdgeType,
    nodes: Vec<NodeIndex>,
}

impl HyperEdge
{
    pub(crate) fn edge_type(&self) -> &EdgeType
    {
        &self.edge_type
    }

    pub(crate) fn nodes(&self) -> &[NodeIndex]
    {
        &self.nodes
    }
}

#[derive(Hash, Clone, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub(crate) enum Node
{
    Technician(TechnicianId),
    WorkOrder(WorkOrderNumber),
    Activity(ActivityNode),
    Period(Period),
    Skill(Skill),
    Day(NaiveDate),
}

#[derive(Hash, Clone, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub(crate) struct ActivityNode
{
    activity_number: ActivityNumber,
    number_of_people: NumberOfPeople,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub enum EdgeType
{
    /// Date specific
    Assign(Option<(StartTime, FinishTime)>),

    /// FORMAT
    /// `vec![$activity, @technicians, @days]`
    Available,
    Exclude,
    BasicStart,

    Contains,
    Requires,
    StartStart,
    FinishStart,
    /// Has skill
    HasSkill,
}

#[derive(Debug)]
pub struct ScheduleGraph
{
    /// Nodes of the problem
    nodes: Vec<Node>,

    /// Hyperedges to handle all the complex interactions
    hyperedges: Vec<HyperEdge>,

    /// Adjacency list
    /// To use this you access with a `NodeIndex` and the
    /// then you get a list of hyperedges-given by `EdgeIndex`-that
    /// this node is a part of. These `EdgeIndex`s can then
    /// be used to find the associated `HyperEdge` with
    /// `ScheduleGraph::hyperedges`.
    incidence_list: Vec<Vec<EdgeIndex>>,

    /// Indices to look up nodes
    technician_indices: HashMap<TechnicianId, NodeIndex>,
    work_order_indices: HashMap<WorkOrderNumber, NodeIndex>,
    period_indices: HashMap<Period, NodeIndex>,
    skill_indices: HashMap<Skill, NodeIndex>,
    day_indices: BTreeMap<NaiveDate, NodeIndex>,
}

/// Public methods
impl ScheduleGraph
{
    pub fn new() -> Self
    {
        Self {
            nodes: vec![],
            hyperedges: vec![],
            incidence_list: vec![],
            technician_indices: HashMap::new(),
            work_order_indices: HashMap::new(),
            period_indices: HashMap::new(),
            skill_indices: HashMap::new(),
            day_indices: BTreeMap::new(),
        }
    }

    pub(crate) fn nodes(&self) -> &[Node]
    {
        &self.nodes
    }

    pub(crate) fn hyperedges(&self) -> &[HyperEdge]
    {
        &self.hyperedges
    }

    pub(crate) fn incidence_list(&self) -> &[Vec<EdgeIndex>]
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
impl ScheduleGraph
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

        let day_node_index = *self
            .day_indices
            .get(&work_order.basic_start())
            .ok_or(ScheduleGraphErrors::DayMissing)?;

        // Crucial lesson! This cannot come first! You learned something great here!
        let work_order_node_index = match self
            .work_order_indices
            .entry(work_order.work_order_number())
        {
            Entry::Vacant(_new_work_order) => {
                self.add_node(Node::WorkOrder(work_order.work_order_number()))
            }
            Entry::Occupied(_already_inserted_work_order) => {
                return Err(ScheduleGraphErrors::WorkOrderDuplicate);
            }
        };

        let _basic_start_edge_index = self.add_edge(
            EdgeType::BasicStart,
            vec![work_order_node_index, day_node_index],
        );

        let mut previous_activity_node = usize::MAX;
        let activity_relations = work_order.activity_relations();
        for (activity_index, operation) in work_order.activities().iter().enumerate() {
            let activity_node_index = self.add_node(Node::Activity(ActivityNode {
                activity_number: operation.operations_number(),
                number_of_people: operation.number_of_people(),
            }));
            let skill_node_index = *self
                .skill_indices
                .get(&operation.skill())
                .ok_or(ScheduleGraphErrors::SkillMissing)?;

            self.add_edge(
                EdgeType::Contains,
                vec![work_order_node_index, activity_node_index],
            );
            self.add_edge(
                EdgeType::Requires,
                vec![activity_node_index, skill_node_index],
            );

            if activity_index != 0 {
                match activity_relations[activity_index - 1] {
                    ActivityRelation::StartStart => self.add_edge(
                        EdgeType::StartStart,
                        vec![previous_activity_node, activity_node_index],
                    ),
                    ActivityRelation::FinishStart => self.add_edge(
                        EdgeType::FinishStart,
                        vec![previous_activity_node, activity_node_index],
                    ),
                    ActivityRelation::Postpone(_time_delta) => todo!(),
                };
            };
            previous_activity_node = activity_node_index;
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
    ) -> Result<NodeIndex, ScheduleGraphErrors>
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
                .ok_or(ScheduleGraphErrors::DayMissing)?;

            single_availability.push(*day_node);
        }

        let technician_id = self.add_node(Node::Technician(technician.id()));

        let mut edges = vec![technician_id];
        edges.extend(skills);
        edges.extend(single_availability);

        let availability_edge = self.add_edge(EdgeType::Available, edges);

        Ok(availability_edge)
    }
}

/// Public API to add [`HyperEdges`] to the graph
impl ScheduleGraph
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
        let worker = self
            .technician_indices
            .get(&worker)
            .ok_or(ScheduleGraphErrors::WorkerMissing)?;
        let work_order = self
            .work_order_indices
            .get(&work_order)
            .ok_or(ScheduleGraphErrors::WorkOrderMissing)?;
        let date = self
            .period_indices
            .get(&date)
            .ok_or(ScheduleGraphErrors::PeriodMissing)?;

        let hyperedge = HyperEdge {
            edge_type: EdgeType::Assign(None),
            nodes: vec![*worker, *work_order, *date],
        };

        self.hyperedges.push(hyperedge);
        Ok(self.hyperedges.len() - 1)
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
                self.day_indices
                    .get(naive_date)
                    .ok_or(ScheduleGraphErrors::DayMissing)?,
            );
        }

        let mut technician_node_indices = vec![];
        'technician: for technician_id in &technicians {
            let technician_node_index = self
                .technician_indices
                .get(technician_id)
                .ok_or(ScheduleGraphErrors::WorkerMissing)?;
            technician_node_indices.push(technician_node_index);

            for availability_hyperedge in
                self.incidence_list[*technician_node_index]
                    .iter()
                    .filter(|&&hyperedge_index| {
                        matches!(
                            self.hyperedges[hyperedge_index].edge_type,
                            EdgeType::Available
                        )
                    })
            {
                match self.hyperedges[*availability_hyperedge].edge_type {
                    // You have to cover the shift with days. That is the most fundamental here.
                    EdgeType::Available => {
                        let availability_nodes = &self.hyperedges[*availability_hyperedge].nodes;

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
        let work_order_node_index = self
            .work_order_indices
            .get(&work_order_number)
            .ok_or(ScheduleGraphErrors::WorkOrderMissing)?;

        // TODO - [ ] Make a `nodes_in_hyperedge(self, edge_id) -> Vec<Nodes>` method.
        let activity_node_index = self
            .incidence_list
            .get(*work_order_node_index)
            .ok_or(ScheduleGraphErrors::WorkOrderMissing)?
            .iter()
            .find_map(|&hyperedge_index| {
                self.hyperedges[hyperedge_index]
                    .nodes
                    .iter()
                    .find(|&&node_index| match &self.nodes[node_index] {
                        Node::Activity(activity) => activity.activity_number == activity_number,
                        _ => false,
                    })
            })
            .ok_or(ScheduleGraphErrors::ActivityMissing)?;

        if let Node::Activity(activity) = &self.nodes[*activity_node_index]
            && technicians.len() > activity.number_of_people as usize
        {
            return Err(ScheduleGraphErrors::ActivityExceedNumberOfPeople);
        }

        let mut final_nodes_in_hyperedge = vec![*activity_node_index];
        final_nodes_in_hyperedge.extend(technician_node_indices);
        final_nodes_in_hyperedge.extend(date_node_indices);

        // TODO [ ] - Add `Day`s as well.
        Ok(self.add_edge(
            EdgeType::Assign(Some(start_and_finish_time)),
            final_nodes_in_hyperedge,
        ))
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
            .iter()
            .any(|e| e == &Node::Period(period_start_date.clone()))
        {
            return Err(ScheduleGraphErrors::PeriodMissing);
        }
        let assignment_hyper_edges = self
            .hyperedges
            .iter()
            .enumerate()
            .filter(|e| matches!(e.1.edge_type, EdgeType::Assign(_)))
            .collect::<Vec<_>>();

        let mut edges = vec![];
        for (edge_index, hyper_edge) in &assignment_hyper_edges {
            for nodes in &hyper_edge.nodes {
                match self.nodes[*nodes] {
                    Node::Period(ref period) => {
                        if period == &period_start_date {
                            edges.push(*edge_index)
                        }
                    }
                    Node::Day(naive_date) => {
                        if period_start_date.start_date().date_naive() <= naive_date
                            && naive_date
                                < (period_start_date.start_date().date_naive() + Duration::days(13))
                        {
                            edges.push(*edge_index)
                        }
                    }
                    // We are only interested in the time of the assignment. `Worker` and
                    // `WorkOrder` belong in a different method.
                    _ => (),
                }
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

        Ok(self.add_edge(EdgeType::HasSkill, vec![*worker, *skill]))
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

        Ok(self.add_edge(EdgeType::Exclude, final_nodes_in_hyperedge))
    }
}

/// Private methods.
///
/// [`NodeIndex`] and [`EdgeIndex`] are not allowed to be a part of the
/// public API of the type. The graph should only expose domain types
/// found in `ordinator-scheduling-environment`
impl ScheduleGraph
{
    fn add_node(&mut self, node: Node) -> NodeIndex
    {
        // This is the next element as `len()` is one larger than the last index
        let node_index = self.nodes.len();
        let none_checker = match node {
            Node::Technician(worker) => self.technician_indices.insert(worker, node_index),
            Node::WorkOrder(work_order) => self.work_order_indices.insert(work_order, node_index),
            Node::Period(naive_date) => self.period_indices.insert(naive_date, node_index),
            Node::Skill(skills) => self.skill_indices.insert(skills, node_index),
            Node::Activity(ref a) => {
                debug!(target: "developer", activity = ?a, "No node index for `Activities`");
                None
            }
            Node::Day(naive_date) => self.day_indices.insert(naive_date, node_index),
        };
        assert!(none_checker.is_none());

        self.incidence_list.push(vec![]);

        // node is added `Vec<Nodes>`
        self.nodes.push(node);
        node_index
    }

    fn add_edge(&mut self, edge_type: EdgeType, nodes: Vec<NodeIndex>) -> EdgeIndex
    {
        let edge_index = self.hyperedges.len();

        for node_index in &nodes {
            self.incidence_list[*node_index].push(edge_index);
        }
        let hyper_edge = HyperEdge { edge_type, nodes };
        self.hyperedges.push(hyper_edge);
        edge_index
    }
}
impl Default for ScheduleGraph
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
    use ordinator_scheduling_environment::work_order::WorkOrder;
    use ordinator_scheduling_environment::work_order::WorkOrderNumber;
    use ordinator_scheduling_environment::work_order::operation::Operation;
    use ordinator_scheduling_environment::worker_environment::availability::Availability;
    use ordinator_scheduling_environment::worker_environment::resources::Skill;
    use ordinator_scheduling_environment::worker_environment::worker::Technician;

    use super::HyperEdge;
    use super::Node;
    use super::ScheduleGraph;
    use crate::schedule_graph::EdgeType;
    use crate::schedule_graph::Period;
    use crate::schedule_graph::ScheduleGraphErrors;

    #[test]
    fn test_schedule_graph_new()
    {
        let mut schedule_graph = ScheduleGraph::new();

        let date = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        let index_worker = schedule_graph.add_node(Node::Technician(1234));
        let index_workorder = schedule_graph.add_node(Node::WorkOrder(WorkOrderNumber(1122334455)));
        let index_period = schedule_graph
            .add_period(Period::from_start_date(date))
            .unwrap();

        assert!(schedule_graph.nodes[index_worker] == Node::Technician(1234));
        assert!(
            schedule_graph.nodes[index_workorder] == Node::WorkOrder(WorkOrderNumber(1122334455))
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
        let mut schedule_graph = ScheduleGraph::new();

        let _skill_node_id = schedule_graph.add_node(Node::Skill(Skill::MtnMech));

        let basic_start_date = NaiveDate::from_ymd_opt(2025, 1, 13).unwrap();
        let work_order = WorkOrder::new(
            1122334455,
            basic_start_date,
            vec![
                Activity::new(10, 1, Skill::MtnMech),
                Activity::new(20, 1, Skill::MtnMech),
                Activity::new(30, 1, Skill::MtnMech),
            ],
        )
        .unwrap();

        assert_eq!(
            schedule_graph.add_work_order(&work_order),
            Err(ScheduleGraphErrors::DayMissing)
        );

        let _period_node_id = schedule_graph
            .add_period(Period::from_start_date(basic_start_date))
            .unwrap();
        let work_order_node_id = schedule_graph
            .add_work_order(&work_order)
            .expect("Could not add work order");

        assert_eq!(
            schedule_graph.nodes[work_order_node_id],
            Node::WorkOrder(WorkOrderNumber(1122334455))
        );

        // let neighbors = schedule_graph..neighbors(node_id).collect::<Vec<_>>();

        assert_eq!(
            schedule_graph.nodes[work_order_node_id + 1],
            Node::Activity(crate::schedule_graph::ActivityNode {
                activity_number: 10,
                number_of_people: 1
            })
        );
        assert_eq!(
            schedule_graph.nodes[work_order_node_id + 2],
            Node::Activity(crate::schedule_graph::ActivityNode {
                activity_number: 20,
                number_of_people: 1
            })
        );
        assert_eq!(
            schedule_graph.nodes[work_order_node_id + 3],
            Node::Activity(crate::schedule_graph::ActivityNode {
                activity_number: 30,
                number_of_people: 1
            })
        );

        let _edge_index = schedule_graph.incidence_list[work_order_node_id + 1]
            .iter()
            .find(|e| {
                schedule_graph.hyperedges[**e]
                    == HyperEdge {
                        edge_type: EdgeType::FinishStart,
                        nodes: vec![work_order_node_id + 1, work_order_node_id + 2],
                    }
            })
            .unwrap();
        let _edge_index = schedule_graph.incidence_list[work_order_node_id + 2]
            .iter()
            .find(|e| {
                schedule_graph.hyperedges[**e]
                    == HyperEdge {
                        edge_type: EdgeType::FinishStart,
                        nodes: vec![work_order_node_id + 2, work_order_node_id + 3],
                    }
            })
            .unwrap();
        assert!(
            !schedule_graph.incidence_list[work_order_node_id + 3]
                .iter()
                .any(|e| {
                    schedule_graph.hyperedges[*e]
                        == HyperEdge {
                            edge_type: EdgeType::FinishStart,
                            nodes: vec![work_order_node_id + 3, work_order_node_id + 4],
                        }
                })
        );

        let basic_start_day_node_id = *schedule_graph.day_indices.get(&basic_start_date).unwrap();

        dbg!(
            &schedule_graph.incidence_list,
            basic_start_day_node_id,
            work_order_node_id,
            &schedule_graph.incidence_list[work_order_node_id],
            &schedule_graph.day_indices,
        );

        let work_order_edge_ids = &schedule_graph.incidence_list[work_order_node_id];

        for edge_id in work_order_edge_ids {
            let hyper_edge = &schedule_graph.hyperedges[*edge_id];
            let edge_type = &hyper_edge.edge_type;
            let nodes = &hyper_edge.nodes;
            match edge_type {
                EdgeType::Assign(_) => todo!(),
                EdgeType::Available => todo!(),
                EdgeType::BasicStart => {
                    assert_eq!(basic_start_day_node_id, nodes[1]);
                    assert_eq!(work_order_node_id, nodes[0]);
                }
                EdgeType::Contains => {
                    assert_eq!(work_order_node_id, nodes[0]);
                }
                EdgeType::Requires => todo!(),
                EdgeType::StartStart => todo!(),
                EdgeType::FinishStart => todo!(),
                EdgeType::Exclude => todo!(),
                EdgeType::HasSkill => todo!(),
            }
        }

        // assert!(day_node == *basic_start_day_node);
        // assert_eq!(period_node_incidence, period_node_id);
    }

    #[test]
    fn test_add_technician()
    {
        let mut schedule_graph = ScheduleGraph::new();

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

        schedule_graph.add_node(Node::Skill(Skill::MtnMech));

        schedule_graph
            .add_period(Period::from_start_date(start.date()))
            .unwrap();

        let availability = Availability::new(start, end);

        schedule_graph
            .add_technician(technician, availability)
            .unwrap();

        assert_eq!(schedule_graph.nodes[0], Node::Skill(Skill::MtnMech));

        for index in 1..=14 {
            let date = start.date();
            assert_eq!(
                schedule_graph.nodes[index],
                Node::Day(date + Duration::days((index - 1) as i64))
            );
        }

        assert_eq!(
            schedule_graph.nodes[15],
            Node::Period(Period::from_start_date(start.date()))
        );

        // TODO [ ] - This should be made into a method for retriving the correct
        // indices
        assert_eq!(
            schedule_graph.hyperedges[0].nodes,
            vec![16, 0, 1, 2, 3, 4, 5, 6, 7]
        );

        assert_eq!(schedule_graph.incidence_list[16], vec![0]);
        assert_eq!(schedule_graph.incidence_list[0], vec![0]);
        assert_eq!(schedule_graph.incidence_list[1], vec![0]);
        assert_eq!(schedule_graph.incidence_list[2], vec![0]);
        assert_eq!(schedule_graph.incidence_list[3], vec![0]);
        assert_eq!(schedule_graph.incidence_list[4], vec![0]);
        assert_eq!(schedule_graph.incidence_list[5], vec![0]);
        assert_eq!(schedule_graph.incidence_list[6], vec![0]);
        assert_eq!(schedule_graph.incidence_list[7], vec![0]);

        // Note: This test needs the schedule graph to have the required skills
        // and days first schedule_graph.add_technician(technician,
        // availability);
    }

    #[test]
    fn test_neighbors()
    {
        let mut schedule_graph = ScheduleGraph::new();

        let date = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        let technician_node_1 = Node::Technician(1234);
        let technician_node_index_1 = schedule_graph.add_node(technician_node_1.clone());
        let work_order_node_1 = Node::WorkOrder(1122334455);
        let work_order_node_index_1 = schedule_graph.add_node(work_order_node_1.clone());
        let period_node_1 = Node::Period(Period::from_start_date(date));
        let period_node_index_1 = schedule_graph.add_node(period_node_1.clone());

        assert!(schedule_graph.nodes[technician_node_index_1] == technician_node_1);
        assert!(schedule_graph.nodes[work_order_node_index_1] == work_order_node_1);
        assert!(schedule_graph.nodes[period_node_index_1] == period_node_1);

        // Using builder to make complex edges will become crucial for the
        // system to function correctly.
        let assignment_edge_index_0 = schedule_graph
            .add_assignment_work_order(1234, 1122334455, Period::from_start_date(date))
            .unwrap();

        let technician_node_2 = Node::Technician(1236);
        let technician_node_index_2 = schedule_graph.add_node(technician_node_2.clone());
        let work_order_node_2 = Node::WorkOrder(1122334456);
        let work_order_node_index_2 = schedule_graph.add_node(work_order_node_2.clone());

        assert!(schedule_graph.nodes[technician_node_index_2] == technician_node_2);
        assert!(schedule_graph.nodes[work_order_node_index_2] == work_order_node_2);
        assert!(schedule_graph.nodes[period_node_index_1] == period_node_1);
        let assignment_edge_index_1 = schedule_graph
            .add_assignment_work_order(1236, 1122334456, Period::from_start_date(date))
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
        let mut schedule_graph = ScheduleGraph::new();

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
        let mut schedule_state = ScheduleGraph::new();

        let period_1 = Period::from_start_date(NaiveDate::from_ymd_opt(2025, 1, 13).unwrap());
        let period_2 = Period::from_start_date(NaiveDate::from_ymd_opt(2025, 1, 27).unwrap());
        let period_3 = Period::from_start_date(NaiveDate::from_ymd_opt(2025, 2, 10).unwrap());

        let _node_id = schedule_state.add_period(period_1).unwrap();
        let _node_id = schedule_state.add_period(period_2).unwrap();
        let _node_id = schedule_state.add_period(period_3).unwrap();

        let node_id = schedule_state.add_period(period_3);

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
            .iter()
            .filter(|&e| matches!(e, Node::Day(_)))
            .collect::<HashSet<_>>();

        let vec_days = schedule_state
            .nodes
            .iter()
            .filter(|&e| matches!(e, Node::Day(_)))
            .collect::<Vec<_>>();

        assert_eq!(hash_set_days.len(), vec_days.len())
    }

    #[test]
    fn test_multi_directional_hypergraph()
    {
        let mut schedule_graph = ScheduleGraph::new();

        let node_0 = Node::WorkOrder(1111990000);
        let node_1 = Node::WorkOrder(1111990001);
        let node_2 = Node::WorkOrder(1111990002);
        let node_3 = Node::WorkOrder(1111990003);
        let node_4 = Node::WorkOrder(1111990004);
        let node_5 = Node::WorkOrder(1111990005);
        let node_6 = Node::WorkOrder(1111990006);
        let node_7 = Node::WorkOrder(1111990007);

        let node_index_0 = schedule_graph.add_node(node_0);
        let node_index_1 = schedule_graph.add_node(node_1);
        let node_index_2 = schedule_graph.add_node(node_2);
        let node_index_3 = schedule_graph.add_node(node_3);
        let node_index_4 = schedule_graph.add_node(node_4);
        let node_index_5 = schedule_graph.add_node(node_5);
        let node_index_6 = schedule_graph.add_node(node_6);
        let node_index_7 = schedule_graph.add_node(node_7);

        let edge_index_0 = schedule_graph.add_edge(EdgeType::Assign(None), vec![0, 2, 4, 6]);
        let edge_index_1 = schedule_graph.add_edge(EdgeType::Assign(None), vec![1, 3, 5, 7]);
        let edge_index_2 = schedule_graph.add_edge(EdgeType::Assign(None), vec![0, 3, 6]);

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
        let mut schedule_graph = ScheduleGraph::new();

        let basic_start_date = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        let work_order = WorkOrder::new(1111990000, basic_start_date, vec![]).unwrap();

        let period = Period::from_start_date(basic_start_date);

        let period_node_index = schedule_graph.add_period(period).unwrap();
        let work_order_node_index = schedule_graph.add_work_order(&work_order).unwrap();

        let exclusion_edge_index = schedule_graph.add_exclusion(&1111990000, &period).unwrap();

        assert_eq!(
            schedule_graph.hyperedges[1],
            HyperEdge {
                edge_type: EdgeType::Exclude,
                nodes: vec![
                    work_order_node_index,
                    period_node_index,
                    0,
                    1,
                    2,
                    3,
                    4,
                    5,
                    6,
                    7,
                    8,
                    9,
                    10,
                    11,
                    12,
                    13,
                ]
            }
        );

        dbg!(
            schedule_graph
                .hyperedges
                .get(schedule_graph.incidence_list[work_order_node_index][0])
        );
        dbg!(
            schedule_graph
                .hyperedges
                .get(schedule_graph.incidence_list[work_order_node_index][1])
        );

        assert!(
            schedule_graph.incidence_list[work_order_node_index].contains(&exclusion_edge_index)
        );
        assert!(schedule_graph.incidence_list[period_node_index].contains(&exclusion_edge_index));
    }

    #[test]
    fn test_add_assignment_activity()
    {
        let mut schedule_graph = ScheduleGraph::new();

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
                Activity::new(10, 2, Skill::MtnMech), // Activity 10, 2 hours, MtnMech skill
                Activity::new(20, 3, Skill::MtnElec), // Activity 20, 3 hours, MtnElec skill
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
        let availability_1 = Availability::new(availability_start_0, availability_end_0);
        let availability_2 = Availability::new(availability_start_1, availability_end_1);
        let availability_3 = Availability::new(availability_start_0, availability_end_0);

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
            vec![1001, 1002],         // technician_ids
            1122334455,               // work_order_number
            10,                       // activity_number
            vec![basic_start_date_0], // days
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
                vec![1001, 1003],         // technician_ids
                1122334455,               // work_order_number
                10,                       // activity_number
                vec![basic_start_date_0], // days
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
        assert!(matches!(hyperedge.edge_type, EdgeType::Assign(Some(_))));

        // Should contain activity + 2 technicians + 1 day = 4 nodes
        assert_eq!(hyperedge.nodes.len(), 4); // activity + 2 technicians + 1 day

        // Verify both technician nodes are in the assignment
        let technician_1_node_id = *schedule_graph.technician_indices.get(&1001).unwrap();
        let technician_3_node_id = *schedule_graph.technician_indices.get(&1003).unwrap();
        assert!(hyperedge.nodes.contains(&technician_1_node_id));
        assert!(hyperedge.nodes.contains(&technician_3_node_id));

        // Verify the assignment shows up in both technicians' incidence lists
        assert!(schedule_graph.incidence_list[technician_1_node_id].contains(&assignment_edge));
        assert!(schedule_graph.incidence_list[technician_3_node_id].contains(&assignment_edge));

        // Verify the activity node is in the assignment
        let day_node_id = *schedule_graph.day_indices.get(&basic_start_date_0).unwrap();
        assert!(hyperedge.nodes.contains(&day_node_id));
    }
}
