use std::path::Path;
use std::sync::Arc;

use chrono::TimeZone;
use chrono::Utc;
use ordinator_orchestrator::Asset;
use ordinator_scheduling_environment::SchedulingEnvironment;
use ordinator_scheduling_environment::materials::MaterialRepo;
use ordinator_scheduling_environment::materials::MaterialToPeriod;
use ordinator_scheduling_environment::time_environment::create_time_environment;
use ordinator_scheduling_environment::work_order::WorkOrderPolicies;
use ordinator_scheduling_environment::work_order::WorkOrdersBuilder;
use ordinator_scheduling_environment::worker_environment::ActorSpecificationBuilder;
use ordinator_scheduling_environment::worker_environment::TimeInput;

pub fn load_scheduling_environment(
    work_order_builder: fn(WorkOrdersBuilder) -> WorkOrdersBuilder,
    worker_builder: fn(ActorSpecificationBuilder) -> ActorSpecificationBuilder,
) -> Arc<std::sync::Mutex<SchedulingEnvironment>>
{
    let time_input = TimeInput {
        number_of_periods: 52,
        number_of_days: 120,
    };

    let time_environment = create_time_environment(
        Utc.with_ymd_and_hms(2025, 1, 13, 7, 0, 0).unwrap(),
        &time_input,
    );

    let work_order_policies =
        "temp_scheduling_environment_database/work_order_policies/work_order_policies_df.toml";

    let path_to_work_order_policies =
        Path::new(env!("CARGO_MANIFEST_DIR")).join(work_order_policies);

    let contents = std::fs::read_to_string(path_to_work_order_policies).unwrap();

    let work_order_policies: WorkOrderPolicies =
        toml::from_str(&contents).expect("Could not read WorkOrderPolicies");

    let material_to_period =
        "temp_scheduling_environment_database/material_repo/material_to_period.toml";
    let path_to_material_to_period = Path::new(env!("CARGO_MANIFEST_DIR")).join(material_to_period);

    let material_to_period_string = std::fs::read_to_string(path_to_material_to_period).unwrap();

    let material_to_period: MaterialToPeriod = toml::from_str(&material_to_period_string).unwrap();

    let material_repo = MaterialRepo::new(material_to_period);

    SchedulingEnvironment::builder()
        .add_actor_specification(Asset::Test, worker_builder)
        .work_order_policies(work_order_policies)
        .material_repo(material_repo)
        .work_orders_builder(work_order_builder)
        .time_environment(time_environment)
        .build()
        .expect("Could not build SchedulingEnvironment")
}
