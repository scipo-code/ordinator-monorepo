use std::collections::HashMap;

use ordinator_orchestrator_actor_traits::SupervisorInterface;
use ordinator_orchestrator_actor_traits::delegate::Delegate;
use ordinator_scheduling_environment::Asset;
use ordinator_scheduling_environment::SchedulingEnvironment;
use ordinator_scheduling_environment::work_order::WorkOrderActivity;
use ordinator_scheduling_environment::work_order::WorkOrderNumber;
use ordinator_scheduling_environment::worker_environment::WorkerEnvironment;
use ordinator_scheduling_environment::worker_environment::resources::Id;
use ordinator_scheduling_environment::worker_environment::resources::Resources;

#[derive(Clone)]
struct TestSystemSolution<Zs: SupervisorInterface + Clone>
{
    supervisor_solution: Zs,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SupervisorSolution
{
    pub(crate) operational_state_machine: HashMap<(Id, WorkOrderActivity), Delegate>,
}

impl SupervisorInterface for SupervisorSolution
{
    fn delegated_tasks(
        &self,
        _operational_agent: &Id,
    ) -> std::collections::HashSet<WorkOrderActivity>
    {
        todo!()
    }

    fn delegates_for_agent(&self, _operational_agent: &Id) -> HashMap<WorkOrderActivity, Delegate>
    {
        todo!()
    }

    fn count_delegate_types(&self, _operational_agent: &Id) -> (u64, u64, u64)
    {
        todo!()
    }
}

// What should you do here? I believe that I should refactor the
// trait. You already knew that this would be a problem and you
// hated it the first time that you made it. Now it is time to
// do this again. I believe that this will have to do in this
// case.

#[test]
fn start_operational_actor()
{
    let operational_id = Id::new(
        "OPERATIONAL_TEST",
        vec![Resources::MtnMech],
        vec![Asset::Test],
    );
    let operational_state_machine = HashMap::from([(
        (operational_id.clone(), (WorkOrderNumber(1001), 10)),
        Delegate::Assign,
    )]);

    let supervisor_solution = SupervisorSolution {
        operational_state_machine,
    };

    // TODO [ ] 2025-07-08 Make a `builder` for the `SystemSolution`.
    // You need to construct a builder for the `SystemSolution` as well.
    let _system_solution = TestSystemSolution {
        supervisor_solution,
    };

    // How do we make a `WorkerEnvironment`. Do we want to rely on files? No! We do
    // not want to rely on raw database JSON/BSON files directly.
    // QUESTION [ ] Should you even be using the `SchedulingEnvironment`? No I do
    // not think that is the best approach here. What other thing could you do?
    // I think that making the a builder for the Actor is the best approach. But
    // how should that look like?
    //
    // TODO [ ] build an OperationalActor and a SupervisorSolution.
    let worker_environment = WorkerEnvironment::builder()
        .actor_environment(Asset::Test)
        // What should be done here? I think that the best approach is to make
        // the system work.
        .unwrap()
        .build();

    // You should stop now. Spend a good evening with Emil, and then work in the
    // evening instead.
    let scheduling_environment = SchedulingEnvironment::builder()
        .worker_environment(worker_environment)
        .work_orders_builder(|wo_builder| {
            wo_builder
                .work_order_builder(WorkOrderNumber(1001), |f| {
                    &mut f.main_work_center(Resources::MtnMech).operations_builder(
                        10,
                        Resources::MtnMech,
                        |f| f.operation_info(|f| f.work_remaining(10.0)),
                    )
                })
                .work_order_builder(WorkOrderNumber(1002), |f| {
                    &mut f.main_work_center(Resources::MtnMech).operations_builder(
                        10,
                        Resources::MtnMech,
                        |f| f.operation_info(|f| f.work_remaining(5.0)),
                    )
                })
        })
        .build();

    // Get it to work first, then change the API
    let _scheduling_environment_guard = scheduling_environment.lock().unwrap();

    // let operational_actor:
    // Actor<OperationalRequestMessage,OperationalResponseMessage,
    // OperationalAlgorithm<TestSystemSolution>> = Actor::builder()
    // .agent_id(operational_id.clone())  .configurations(k)
    // How would you solve this? The best approach is to get it to run first.
    // And then you can think about changing the API of the program.
    // TODO [ ] 2025-07-08 make the `OperationalActor` run.
    //  .algorithm(|a|
    //  .id(Id::from).parameters_and_solution(scheduling_environment))

    // .build()
    // [ ] You should spend the next 30 minutes on this and then move on to talk
    // with Emil.
}
