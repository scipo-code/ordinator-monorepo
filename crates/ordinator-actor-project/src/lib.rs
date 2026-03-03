pub mod algorithm;
pub mod messages;

use std::fmt::Debug;
use std::ops::Deref;
use std::ops::DerefMut;

use algorithm::ProjectAlgorithm;
use algorithm::project_solution::ProjectSolution;
use messages::ProjectRequestMessage;
use messages::ProjectResponseMessage;
use ordinator_actor_core::Actor;
use ordinator_orchestrator_actor_traits::CommandHandler;
use ordinator_orchestrator_actor_traits::SystemSolutions;

pub struct ProjectActor<Ss: Debug>(
    Actor<ProjectRequestMessage, ProjectResponseMessage, ProjectAlgorithm<Ss>>,
)
where
    Ss: SystemSolutions<Project = ProjectSolution>,
    Actor<ProjectRequestMessage, ProjectResponseMessage, ProjectAlgorithm<Ss>>:
        CommandHandler<ProjectRequestMessage, ProjectResponseMessage>;

impl<Ss> Deref for ProjectActor<Ss>
where
    Ss: SystemSolutions<Project = ProjectSolution> + Debug,
    Actor<ProjectRequestMessage, ProjectResponseMessage, ProjectAlgorithm<Ss>>:
        CommandHandler<ProjectRequestMessage, ProjectResponseMessage>,
{
    type Target = Actor<ProjectRequestMessage, ProjectResponseMessage, ProjectAlgorithm<Ss>>;

    fn deref(&self) -> &Self::Target
    {
        &self.0
    }
}

impl<Ss: Debug> DerefMut for ProjectActor<Ss>
where
    Ss: SystemSolutions<Project = ProjectSolution>,
    Actor<ProjectRequestMessage, ProjectResponseMessage, ProjectAlgorithm<Ss>>:
        CommandHandler<ProjectRequestMessage, ProjectResponseMessage>,
{
    fn deref_mut(&mut self) -> &mut Self::Target
    {
        &mut self.0
    }
}

