// ISSUE #000 Come back to this later on.
// use std::sync::Arc;
// use std::sync::Mutex;

// use anyhow::Result;
// use arc_swap::ArcSwap;
// use ordinator_orchestrator_actor_traits::SystemSolutions;
// use ordinator_scheduling_environment::SchedulingEnvironment;
// ESSAY:
// I think that I should make sure to start testing and developing on the
// TacticalActor first and then do this. That is a better approach I think
// hmm... No it is not you have to specify the constraints on the
// data structure and then start developing. Otherwise you do not know
// where to make changes to the algorithm.
// struct SystemSolutionTester
// {
//     system_solution: Arc<ArcSwap<dyn SystemSolutions>>,
//     scheduling_environment: Arc<Mutex<SchedulingEnvironment>>,
// }

// impl SystemSolutionTester
// {
//     pub fn test_strategic_actor(&self) -> Result<()>
//     {
//         let a = 1;
//         Ok(())
//     }

//     pub fn test_tactical_actor(&self) -> Result<()>
//     {
//         self.system_solution.load()
//     }
// }

// To make this correctly you will need to inject the state needed. There is
// an issue that the Actor has the state that is needed to test the solution
// I think that the best approach here will be to make the system work with
// independently.
//
// You are getting frustrated! That is exactly what you need here.
