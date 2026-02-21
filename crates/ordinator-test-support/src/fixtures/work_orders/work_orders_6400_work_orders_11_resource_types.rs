use std::fs;

/// Generates complex test data for stress testing the optimization algorithms and their interactions.
/// Focuses on data volume rather than attribute correctness.
use chrono::TimeDelta;
use ordinator_scheduling_environment::work_order::WorkOrdersBuilder;
use ordinator_scheduling_environment::work_order::work_order_info::WorkOrderInfoDetail;
use ordinator_scheduling_environment::work_order::work_order_info::priority::Priority;
use ordinator_scheduling_environment::work_order::work_order_info::revision::Revision;
use ordinator_scheduling_environment::work_order::work_order_info::system_condition::SystemCondition;
use ordinator_scheduling_environment::work_order::work_order_info::work_order_text::WorkOrderText;
use ordinator_scheduling_environment::work_order::work_order_info::work_order_type::WorkOrderType;
use ordinator_scheduling_environment::worker_environment::resources::Skill;

use crate::fixtures::work_orders::WorkOrderData;

/// Builds PhD dataset work orders with the following numbering scheme:
/// - 111199xxxx: normal filler work order without binding status code, vendor, or shutdown
/// - 222299xxxx: normal filler work order with status code modifiers
/// - 333399xxxx: edge case work orders for testing specific aspects
pub fn phd_work_order_builder(mut wo_builder: WorkOrdersBuilder) -> WorkOrdersBuilder
{
    let work_order_json_string =
        fs::read_to_string("./crates/ordinator-test-support/src/fixtures/work_orders/work_orders_6400_work_orders_11_resource_types.json")
            .expect("Test data file is missing");

    let work_order_data: Vec<WorkOrderData> = serde_json::from_str(&work_order_json_string)
        .expect("Could not deserialize the test data JSON.");

    for work_order in work_order_data.iter() {
        wo_builder = wo_builder.work_order_builder(work_order.work_order_number, |wob| {
            let mut wob = wob.main_work_center(Skill::MtnMech);
            for opr in &work_order.operations {
                wob = wob
                    .operations_builder(opr.activity, opr.resource, |ob| {
                        ob.operation_info(|oib| {
                            oib.work_remaining(opr.work_remaining)
                                .work(5.0)
                                .work_actual(5.0)
                        })
                        .operation_dates(|dates| {
                            dates
                                .earliest_start_from_ymd_hms(
                                    opr.early_start.0,
                                    opr.early_start.1,
                                    opr.early_start.2,
                                    opr.early_start.3,
                                    opr.early_start.4,
                                    opr.early_start.5,
                                )
                                .earliest_finish_from_ymd_hms(
                                    opr.early_finish.0,
                                    opr.early_finish.1,
                                    opr.early_finish.2,
                                    opr.early_finish.3,
                                    opr.early_finish.4,
                                    opr.early_finish.5,
                                )
                        })
                        .operation_analytic(|oab| {
                            oab.duration(1.0).preparation_time(opr.preparation)
                        })
                    })
                    .unwrap();
            }
            wob.work_order_info_builder(|woib| {
                woib.priority(Priority::new_int(3))
                    // TODO: Use work order type parameter instead of hardcoded value
                    .work_order_type(WorkOrderType::Wdf(Priority::new_int(3)))
                    .revision(Revision::new("NOSD"))
                    .work_order_text(WorkOrderText {
                        order_system_status: Some("TEST".to_string()),
                        order_user_status: Some("TEST".to_string()),
                        order_description: "Basic Mechnical Job".to_string(),
                        object_description: Some("TEST".to_string()),
                        notes_1: Some("TEST".to_string()),
                        notes_2: Some(1),
                    })
                    .functional_location_from_str("TEST/XX/XX/101")
                    .system_condition(SystemCondition::A)
                    .work_order_info_detail(WorkOrderInfoDetail {
                        subnetwork: "123".to_string(),
                        maintenance_plan: "PLAN TEST".to_string(),
                        planner_group: "TEST_GROUP".to_string(),
                        maintenance_plant: "TEST".to_string(),
                        pm_collective: "TEST".to_string(),
                        room: "TEST_ROOM".to_string(),
                    })
            })
            .work_order_dates_builder(|wodb| {
                wodb.duration(TimeDelta::days(1))
                    .basic_start_from_ymd(
                        work_order.basic_start.0,
                        work_order.basic_start.1,
                        work_order.basic_start.2,
                    )
                    .basic_finish_from_ymd(
                        work_order.basic_finish.0,
                        work_order.basic_finish.1,
                        work_order.basic_finish.2,
                    )
                    .earliest_allowed_start_from_ymd(
                        work_order.easd.0,
                        work_order.easd.1,
                        work_order.easd.2,
                    )
                    .latest_allowed_finish_from_ymd(
                        work_order.lafd.0,
                        work_order.lafd.1,
                        work_order.lafd.2,
                    )
            })
            .work_order_analytic_builder(|woab| {
                woab.user_status_codes(|user| user.smat(work_order.codes.0))
                    .system_status_codes(|system| system.rel(work_order.codes.1))
            })
        });
    }
    wo_builder
}
