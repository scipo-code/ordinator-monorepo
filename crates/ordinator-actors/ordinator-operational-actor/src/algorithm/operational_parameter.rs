use std::collections::HashMap;
// TODO [ ]
// I think that I myself want to control how this type should be displayed. That
// will make the most sense.
//
// NOTE [ ] Should only make it yourself or simply derive it now? I think that
// deriving it now is the best decision that you can make and then simply be
// ready to make a better one when it is needed
//
// NOTE [ ]
// Remember that what you generally want is to have the code put out the
// [`SystemSolution`] that is currently loaded into the program. Not the current
// one.
use std::fmt::Debug;
use std::sync::MutexGuard;

use anyhow::Context;
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
use ordinator_scheduling_environment::worker_environment::OperationalOptions;
use ordinator_scheduling_environment::worker_environment::availability::Availability;
use ordinator_scheduling_environment::worker_environment::resources::ActorCompositeId;

// Again there are here multiple ways of doing things. You should be careful
// I think that the best approach is to put the... You could reformulate
// the [`SchedulingEnvironment`] and then update the precedence relation
// directly from the source. is this a good idea? The other approach is
// to have the code have an additional field here.
//
// You have a dilemma here, in the the choice of turning the
// SchedulingEnvironment into a highly concurrent data structure will allow you
// to remove an insane amount of state duplication. I am not sure that this is
// something that you want. It is?
//
// You have to make a SWAT analysis. From your current view point it seems like
// a good idea. but is it? I am not really sure. You are doing everything right
// here.
//
// I do not think that you should make a highly concurrent SchedulingEnvironment
// yet
pub struct OperationalParameters
{
    pub work_order_parameters: HashMap<WorkOrderActivity, OperationalParameter>,
    pub work_order_activity_relations: HashMap<WorkOrderNumber, Vec<ActivityRelation>>,
    pub availability: Availability,
    pub off_shift_interval: TimeInterval,
    pub break_interval: TimeInterval,
    pub toolbox_interval: TimeInterval,
    pub options: OperationalOptions,
}

impl Debug for OperationalParameters
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        if !f.alternate() {
            return write!(
                f,
                "OperationalParameters{{activities: {}, availability: {:?}, \
                 off_shift: {:?}, break: {:?}, toolbox: {:?}, options: {:?}}}",
                self.work_order_parameters.len(),
                self.availability,
                self.off_shift_interval,
                self.break_interval,
                self.toolbox_interval,
                self.options,
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

        writeln!(f, "    options: {:#?},", self.options,)?;

        write!(f, "}}")
    }
}

// There is something rotten about this function.
impl Parameters for OperationalParameters
{
    type Key = WorkOrderActivity;

    // You should not put it in the Options

    // Do we even want the code to look like this in the first place?
    fn from_source(
        id: &ActorCompositeId,
        scheduling_environment: &MutexGuard<SchedulingEnvironment>,
        // This is not needed. It should always be a part of your SchedulingEnvironment.
        // Yes this is the best approach here.
    ) -> Result<Self>
    {
        let mut work_order_parameters = HashMap::default();
        let mut work_order_activity_relations = HashMap::default();

        for (&work_order_number, work_order) in &scheduling_environment
            .work_orders
            .inner
            .iter()
            .filter(|(_, wo)| wo.released_for_scheduling())
            .collect::<HashMap<_, _>>()
        {
            for activity_number in work_order.activity_numbers() {
                let work_order_activity = (*work_order_number, activity_number);

                let work_remaining = work_order.operation_work_remaining(activity_number);
                let preparation_time = work_order.operation_preparation(activity_number);
                let operational_parameter_option = OperationalParameter::new(
                    work_remaining.context("Could not derive work_remaining")?,
                    preparation_time.context("Could not derive preparation_time")?,
                );

                // Are we mutating this function?
                let operational_parameter = match operational_parameter_option {
                    Some(operational_parameter) => operational_parameter,
                    None => continue,
                };
                ensure!(
                    !operational_parameter.work.is_zero(),
                    "Work for an activity should never be zero in the OperationalActor"
                );

                work_order_parameters.insert(work_order_activity, operational_parameter);
            }
            let activity_relations = work_order.activity_relations();

            work_order_activity_relations.insert(*work_order_number, activity_relations);
        }

        let operational_configuration = &scheduling_environment
            .worker_environment
            .actor_specification
            .get(id.asset())
            .with_context(|| format!("{:#?}", id.asset()))?
            .operational()
            .iter()
            .find(|oca| id.0 == *oca.0)
            .with_context(|| format!("OperationalActor: {:#?} does not exist", id))?;

        // TODO [ ] Start operational for each `Availability`
        Ok(Self {
            work_order_parameters,
            work_order_activity_relations,
            availability: id.2.clone(),
            off_shift_interval: operational_configuration
                .1
                .operational_configuration
                .off_shift_interval
                .clone(),
            break_interval: operational_configuration
                .1
                .operational_configuration
                .break_interval
                .clone(),
            toolbox_interval: operational_configuration
                .1
                .operational_configuration
                .toolbox_interval
                .clone(),
            options: OperationalOptions {
                number_of_removed_activities: operational_configuration
                    .1
                    .operational_options
                    .number_of_removed_activities,
            },
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
        //
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
