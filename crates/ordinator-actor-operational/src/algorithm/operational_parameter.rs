use std::collections::HashMap;
// TODO: Implement custom Display implementation for better control over formatting
// NOTE: Derive Debug for now; replace with custom implementation when needed
// NOTE: Ensure output displays the currently loaded [`SystemSolution`], not previous versions
use std::sync::MutexGuard;

use anyhow::Result;
use anyhow::ensure;
use chrono::TimeDelta;
use colored::Colorize;
use ordinator_orchestrator_actor_traits::Parameters;
use ordinator_scheduling_environment::SchedulingEnvironment;
use ordinator_scheduling_environment::time_environment::TimeInterval;
use ordinator_scheduling_environment::work_order::ActivityRelation;
use ordinator_scheduling_environment::work_order::WorkOrderActivity;
use ordinator_scheduling_environment::work_order::WorkOrderNumber;
use ordinator_scheduling_environment::work_order::operation::Work;
use ordinator_scheduling_environment::worker_environment::availability::Availability;
use ordinator_scheduling_environment::worker_environment::resources::ActorCompositeId;
use ordinator_scheduling_hypergraph::schedule_graph::SchedulingHypergraph;

// Consider refactoring SchedulingEnvironment into a highly concurrent data
// structure to reduce state duplication. For now, maintain current approach
// with separate fields.
pub struct OperationalParameters
{
    pub work_order_parameters: HashMap<WorkOrderActivity, OperationalParameter>,
    pub work_order_activity_relations: HashMap<WorkOrderNumber, Vec<ActivityRelation>>,
    pub availability: Availability,
    pub off_shift_interval: TimeInterval,
    pub break_interval: TimeInterval,
    pub toolbox_interval: TimeInterval,
}

impl std::fmt::Debug for OperationalParameters
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        if !f.alternate() {
            return write!(
                f,
                "OperationalParameters{{activities: {}, availability: {:?}, \
                 off_shift: {:?}, break: {:?}, toolbox: {:?}}}",
                self.work_order_parameters.len(),
                self.availability,
                self.off_shift_interval,
                self.break_interval,
                self.toolbox_interval,
            );
        }

        writeln!(f, "{} {{", "OperationalParameters".yellow())?;

        writeln!(
            f,
            "    {}: {:#?},",
            "work_order_parameters".yellow(),
            self.work_order_parameters.len()
        )?;

        writeln!(
            f,
            "    {}: {:#?},",
            "availability".green(),
            self.availability
        )?;

        writeln!(
            f,
            "    {}: {:#?},",
            "off_shift_interval".green(),
            &self.off_shift_interval,
        )?;

        writeln!(
            f,
            "    {}: {:#?},",
            "break_interval".green(),
            self.break_interval,
        )?;

        writeln!(
            f,
            "    {}: {:#?},",
            "toolbox_interval".green(),
            self.toolbox_interval,
        )?;

        write!(f, "}}")
    }
}

impl Parameters for OperationalParameters
{
    type Key = WorkOrderActivity;

    fn from_scheduling_hypergraph(
        id: &ActorCompositeId,
        scheduling_hypergraph: &MutexGuard<SchedulingHypergraph>,
    ) -> Result<Self>
    {
        let weekly_view = scheduling_hypergraph.extract_weekly_view();

        let mut work_order_parameters = HashMap::default();
        let mut work_order_activity_relations = HashMap::default();

        for (&work_order_number, wo_view) in &weekly_view.work_orders {
            let mut relations = Vec::new();
            for activity in &wo_view.activities {
                let work_order_activity = (work_order_number, activity.activity_number);

                // Use work_remaining as both work and preparation (preparation
                // not available from hypergraph)
                let operational_parameter_option = OperationalParameter::new(
                    activity.work_remaining,
                    Work::from(0.0),
                );

                let operational_parameter = match operational_parameter_option {
                    Some(operational_parameter) => operational_parameter,
                    None => continue,
                };
                ensure!(
                    !operational_parameter.work.is_zero(),
                    "Work for an activity should never be zero in the OperationalActor"
                );

                work_order_parameters.insert(work_order_activity, operational_parameter);

                if let Some(relation) = &activity.relation_to_next {
                    relations.push(relation.clone());
                }
            }

            work_order_activity_relations.insert(work_order_number, relations);
        }

        // Operational configuration (off_shift, break, toolbox intervals) is not
        // available from the hypergraph. Use sensible defaults.
        Ok(Self {
            work_order_parameters,
            work_order_activity_relations,
            availability: id.2.clone(),
            off_shift_interval: TimeInterval::from_hms(0, 0, 0, 0, 0, 1)?,
            break_interval: TimeInterval::from_hms(12, 0, 0, 12, 30, 0)?,
            toolbox_interval: TimeInterval::from_hms(6, 0, 0, 6, 15, 0)?,
        })
    }

    fn create_and_insert_new_parameter(
        &mut self,
        _key: Self::Key,
        _scheduling_environment: MutexGuard<SchedulingEnvironment>,
    )
    {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct OperationalParameter
{
    pub work: Work,
    // TODO: INCLUDE PREPARATION
    pub _preparation: Work,
    pub operation_time_delta: TimeDelta,
    // start_window: DateTime<Utc>,
    // end_window: DateTime<Utc>,
    // pub delegated: Delegate,
    // marginal_fitness: MarginalFitness,
}

impl OperationalParameter
{
    pub fn new(
        work: Work,
        _preparation: Work,
        // start_window: DateTime<Utc>,
        // end_window: DateTime<Utc>,
        // delegated: Delegate,
        // marginal_fitness: MarginalFitness,
    ) -> Option<Self>
    {
        let combined_time = (work + _preparation).in_seconds();
        let operation_time_delta = TimeDelta::new(combined_time, 0).unwrap();
        if work.to_f64() == 0.0 {
            return None;
        }
        if operation_time_delta == TimeDelta::new(0, 0).unwrap() {
            return None;
        }
        Some(Self {
            work,
            _preparation,
            operation_time_delta,
            // start_window,
            // end_window,
            // delegated,
            // marginal_fitness,
        })
    }
}
