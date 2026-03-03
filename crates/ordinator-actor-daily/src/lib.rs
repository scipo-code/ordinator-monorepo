pub mod algorithm;
mod assert_functions;
pub mod messages;
use std::fmt::Debug;
use std::ops::Deref;
use std::ops::DerefMut;

use algorithm::DailyAlgorithm;
use algorithm::daily_solution::DailySolution;
#[allow(unused_imports)]
use assert_functions::DailyAssertions;
use messages::DailyRequestMessage;
use messages::DailyResponseMessage;
use ordinator_actor_core::Actor;
use ordinator_orchestrator_actor_traits::CommandHandler;
use ordinator_orchestrator_actor_traits::SystemSolutions;

pub struct DailyActor<Ss: Debug>(
    Actor<DailyRequestMessage, DailyResponseMessage, DailyAlgorithm<Ss>>,
)
where
    Ss: SystemSolutions<Daily = DailySolution>,
    Actor<DailyRequestMessage, DailyResponseMessage, DailyAlgorithm<Ss>>:
        CommandHandler<DailyRequestMessage, DailyResponseMessage>;

impl<Ss> Deref for DailyActor<Ss>
where
    Ss: SystemSolutions<Daily = DailySolution> + Debug,
    Actor<DailyRequestMessage, DailyResponseMessage, DailyAlgorithm<Ss>>:
        CommandHandler<DailyRequestMessage, DailyResponseMessage>,
{
    type Target = Actor<DailyRequestMessage, DailyResponseMessage, DailyAlgorithm<Ss>>;

    fn deref(&self) -> &Self::Target
    {
        &self.0
    }
}

impl<Ss> DerefMut for DailyActor<Ss>
where
    Ss: SystemSolutions<Daily = DailySolution> + Debug,
    Actor<DailyRequestMessage, DailyResponseMessage, DailyAlgorithm<Ss>>:
        CommandHandler<DailyRequestMessage, DailyResponseMessage>,
{
    fn deref_mut(&mut self) -> &mut Self::Target
    {
        &mut self.0
    }
}

