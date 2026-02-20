// TODO: Revisit this section
// use std::sync::Arc;
// use std::sync::Mutex;

// use anyhow::Result;
// use arc_swap::ArcSwap;
// use ordinator_orchestrator_actor_traits::SystemSolutions;
// use ordinator_scheduling_environment::SchedulingEnvironment;
// Note: Constraints on the data structure should be specified before algorithm development
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

//     pub fn test_project_actor(&self) -> Result<()>
//     {
//         self.system_solution.load()
//     }
// }

// TODO: Inject state for testing; consider making the system work independently
