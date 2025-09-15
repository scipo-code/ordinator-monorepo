use std::sync::Arc;
use std::collections::HashMap;

use chrono::TimeZone;
use chrono::Utc;
use ordinator_scheduling_environment::Asset;
use ordinator_scheduling_environment::SchedulingEnvironment;
use ordinator_scheduling_environment::materials::MaterialRepo;
use ordinator_scheduling_environment::materials::MaterialToPeriod;
use ordinator_scheduling_environment::time_environment::create_time_environment;
use ordinator_scheduling_environment::work_order::WorkOrderPolicies;
use ordinator_scheduling_environment::work_order::WorkOrdersBuilder;
use ordinator_scheduling_environment::work_order::ClusteringWeights;
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

    let work_order_policies = WorkOrderPolicies::builder()
        .operating_time(6)
        .wdf_priority_map({
            let mut map = HashMap::new();
            map.insert("1".to_string(), 500);
            map.insert("2".to_string(), 50);
            map.insert("3".to_string(), 5);
            map.insert("4".to_string(), 1);
            map.insert("5".to_string(), 1);
            map.insert("6".to_string(), 1);
            map.insert("7".to_string(), 1);
            map.insert("8".to_string(), 1);
            map
        })
        .wgn_priority_map({
            let mut map = HashMap::new();
            map.insert("1".to_string(), 500);
            map.insert("2".to_string(), 50);
            map.insert("3".to_string(), 5);
            map.insert("4".to_string(), 1);
            map.insert("5".to_string(), 1);
            map.insert("6".to_string(), 1);
            map.insert("7".to_string(), 1);
            map.insert("8".to_string(), 1);
            map
        })
        .wpm_priority_map({
            let mut map = HashMap::new();
            map.insert('A', 500);
            map.insert('B', 50);
            map.insert('C', 5);
            map.insert('D', 1);
            map
        })
        .vis_priority_map({
            let mut map = HashMap::new();
            map.insert('V', 100);
            map.insert('I', 10);
            map.insert('S', 10);
            map
        })
        .order_type_weights({
            let mut map = HashMap::new();
            map.insert("WDF".to_string(), 10);
            map.insert("WGN".to_string(), 8);
            map.insert("WPM".to_string(), 5);
            map.insert("Other".to_string(), 0);
            map
        })
        .status_weights({
            let mut map = HashMap::new();
            map.insert("SECE".to_string(), 7500);
            map.insert("PCNF_NMAT_SMAT".to_string(), 1500);
            map.insert("AWSC".to_string(), 10000);
            map
        })
        .clustering_weights(ClusteringWeights {
            asset: 10,
            sector: 5,
            system: 2,
            subsystem: 2,
            equipment_tag: 1,
        })
        .build();

    let material_to_period = MaterialToPeriod::builder()
        .nmat(0)
        .smat(0)
        .cmat(2)
        .pmat(3)
        .wmat(3)
        .build();

    let material_repo = MaterialRepo::builder()
        .material_to_period(material_to_period)
        .build();

    SchedulingEnvironment::builder()
        .add_actor_specification(Asset::Test, worker_builder)
        .work_order_policies(work_order_policies)
        .material_repo(material_repo)
        .work_orders_builder(work_order_builder)
        .time_environment(time_environment)
        .build()
        .expect("Could not build SchedulingEnvironment")
}
