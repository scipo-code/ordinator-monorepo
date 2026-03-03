pub mod algorithm;
mod assert_functions;
pub mod messages;

use std::fmt::Debug;
use std::ops::Deref;
use std::ops::DerefMut;

use algorithm::OperationalAlgorithm;
use algorithm::operational_solution::OperationalSolution;
use messages::OperationalRequestMessage;
use messages::OperationalResponseMessage;
use ordinator_actor_core::Actor;
use ordinator_orchestrator_actor_traits::CommandHandler;
use ordinator_orchestrator_actor_traits::SystemSolutions;

pub struct OperationalActor<Ss: Debug>(
    Actor<OperationalRequestMessage, OperationalResponseMessage, OperationalAlgorithm<Ss>>,
)
where
    Ss: SystemSolutions<Operational = OperationalSolution>,
    Actor<OperationalRequestMessage, OperationalResponseMessage, OperationalAlgorithm<Ss>>:
        CommandHandler<OperationalRequestMessage, OperationalResponseMessage>;

impl<Ss> Deref for OperationalActor<Ss>
where
    Ss: SystemSolutions<Operational = OperationalSolution> + Debug,
    Actor<OperationalRequestMessage, OperationalResponseMessage, OperationalAlgorithm<Ss>>:
        CommandHandler<OperationalRequestMessage, OperationalResponseMessage>,
{
    type Target =
        Actor<OperationalRequestMessage, OperationalResponseMessage, OperationalAlgorithm<Ss>>;

    fn deref(&self) -> &Self::Target
    {
        &self.0
    }
}

impl<Ss: Debug> DerefMut for OperationalActor<Ss>
where
    Ss: SystemSolutions<Operational = OperationalSolution>,
    Actor<OperationalRequestMessage, OperationalResponseMessage, OperationalAlgorithm<Ss>>:
        CommandHandler<OperationalRequestMessage, OperationalResponseMessage>,
{
    fn deref_mut(&mut self) -> &mut Self::Target
    {
        &mut self.0
    }
}

