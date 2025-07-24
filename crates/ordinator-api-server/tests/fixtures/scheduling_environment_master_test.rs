use std::path::Path;
use std::sync::Arc;

use chrono::NaiveDate;
use chrono::TimeDelta;
use chrono::TimeZone;
use chrono::Utc;
use ordinator_orchestrator::Asset;
use ordinator_orchestrator::WorkOrderNumber;
use ordinator_scheduling_environment::SchedulingEnvironment;
use ordinator_scheduling_environment::time_environment::create_time_environment;
use ordinator_scheduling_environment::work_order::work_order_info::WorkOrderInfoDetail;
use ordinator_scheduling_environment::work_order::work_order_info::priority::Priority;
use ordinator_scheduling_environment::work_order::work_order_info::revision::Revision;
use ordinator_scheduling_environment::work_order::work_order_info::system_condition::SystemCondition;
use ordinator_scheduling_environment::work_order::work_order_info::work_order_text::WorkOrderText;
use ordinator_scheduling_environment::work_order::work_order_info::work_order_type::WorkOrderType;
use ordinator_scheduling_environment::worker_environment::ActorEnvironment;
use ordinator_scheduling_environment::worker_environment::TimeInput;
use ordinator_scheduling_environment::worker_environment::resources::Resources;

pub fn load_scheduling_environment() -> Arc<std::sync::Mutex<SchedulingEnvironment>>
{
    let asset = Asset::Test;
    let asset_string = asset.to_string().to_lowercase();

    let path = format!(
        "temp_scheduling_environment_database/actor_specifications/actor_specification_{asset_string}.toml",
    );
    let path_to_data = Path::new(env!("CARGO_MANIFEST_DIR")).join(path);

    println!("{}\n{}", path_to_data.display(), line!());
    // println!("{:?}", std::fs::canonicalize(path_to_data.clone()).unwrap());

    // We do not want to test against data files. I think that the best approach
    // here will be to test against something else.
    let worker_environment = ActorEnvironment::builder()
        .actor_environment(Asset::Test, path_to_data)
        // What should be done here? I think that the best approach is to make
        // the system work.
        .unwrap()
        .build();
    // Should you build the actors yourself. Or do something different? I think that
    // the best approach here is to do the same thing again.

    let time_input = TimeInput {
        number_of_periods: 5,
        number_of_days: 42,
    };

    let time_environment = create_time_environment(
        Utc.with_ymd_and_hms(2025, 1, 13, 7, 0, 0).unwrap(),
        &time_input,
    );

    SchedulingEnvironment::builder()
        .worker_environment(worker_environment)
        .work_orders_builder(|wo_builder| {
            wo_builder
                .work_order_builder(WorkOrderNumber(1001), |wob| {
                    wob.main_work_center(Resources::MtnMech)
                        .operations_builder(10, Resources::MtnMech, |ob| {
                            ob.operation_info(|oib| {
                                oib.work_remaining(10.0).work(5.0).work_actual(5.0)
                            })
                            .operation_dates(|dates| {
                                dates
                                    .earliest_start_datetime(
                                        Utc.with_ymd_and_hms(2025, 1, 1, 7, 0, 0).unwrap(),
                                    )
                                    .earliest_finish_datetime(
                                        Utc.with_ymd_and_hms(2025, 1, 2, 7, 0, 0).unwrap(),
                                    )
                            })
                            .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
                        })
                        .work_order_info_builder(|woib| {
                            woib.priority(Priority::new_int(1))
                                .work_order_type(WorkOrderType::Wdf(Priority::new_int(1)))
                                .revision(Revision::new("NOSD"))
                                .work_order_text(WorkOrderText {
                                    order_system_status: Some("TEST".to_string()),
                                    order_user_status: Some("TEST".to_string()),
                                    order_description: "TEST".to_string(),
                                    operation_description: Some("TEST".to_string()),
                                    object_description: Some("TEST".to_string()),
                                    notes_1: Some("TEST".to_string()),
                                    notes_2: Some(1),
                                })
                                .functional_location_from_str("TEST/XX/XX/101")
                                .system_condition(SystemCondition::A)
                                // It is clear that you need a thorough understanding of the whole
                                // maintenance process to be able to develop this system.
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
                                .basic_start_date(NaiveDate::from_ymd_opt(2025, 1, 1).expect("This date is required for constructing a WorkOrderDates object"))
                                .basic_finish_date(NaiveDate::from_ymd_opt(2025, 1, 1).expect("This date is required for constructing a WorkOrderDates object"))
                                .earliest_allowed_start_date(NaiveDate::from_ymd_opt(2025, 1, 1).expect("This date is required for constructing a WorkOrderDates object"))
                                .latest_allowed_finish_date(NaiveDate::from_ymd_opt(2025, 5, 1).expect("This date is required for constructing a WorkOrderDates object"))
                        })
                        .work_order_analytic_builder(|woab| {
                            woab.user_status_codes(|user| user.smat(true).rel(true))
                                .system_status_codes(|system| system.rel(true))

                        })
                })
                .work_order_builder(WorkOrderNumber(1002), |wob| {
                    wob.main_work_center(Resources::MtnMech)
                        .operations_builder(10, Resources::MtnMech, |ob| {
                            ob.operation_info(|oib| {
                                oib.work_remaining(5.0).work(5.0).work_actual(5.0)
                            })
                            .operation_dates(|dates| {
                                dates
                                    .earliest_start_datetime(
                                        Utc.with_ymd_and_hms(2025, 1, 1, 7, 0, 0).unwrap(),
                                    )
                                    .earliest_finish_datetime(
                                        Utc.with_ymd_and_hms(2025, 1, 2, 7, 0, 0).unwrap(),
                                    )
                            })
                            .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
                        })
                        .work_order_info_builder(|woib| {
                            woib.priority(Priority::new_int(1))
                                .work_order_type(WorkOrderType::Wdf(Priority::new_int(1)))
                                .revision(Revision::new("NOSD"))
                                .work_order_text(WorkOrderText {
                                    order_system_status: Some("TEST".to_string()),
                                    order_user_status: Some("TEST".to_string()),
                                    order_description: "TEST".to_string(),
                                    operation_description: Some("TEST".to_string()),
                                    object_description: Some("TEST".to_string()),
                                    notes_1: Some("TEST".to_string()),
                                    notes_2: Some(1),
                                })
                                .functional_location_from_str("TEST/XX/XX/101")
                                .system_condition(SystemCondition::A)
                                // It is clear that you need a thorough understanding of the whole
                                // maintenance process to be able to develop this system.
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
                                .basic_start_date(NaiveDate::from_ymd_opt(2025, 1, 1).expect("This date is required for constructing a WorkOrderDates object"))
                                .basic_finish_date(NaiveDate::from_ymd_opt(2025, 1, 1).expect("This date is required for constructing a WorkOrderDates object"))
                                .earliest_allowed_start_date(NaiveDate::from_ymd_opt(2025, 1, 1).expect("This date is required for constructing a WorkOrderDates object"))
                                .latest_allowed_finish_date(NaiveDate::from_ymd_opt(2025, 5, 1).expect("This date is required for constructing a WorkOrderDates object"))
                        })
                        .work_order_analytic_builder(|woab| {
                            woab.user_status_codes(|user| user.smat(true).rel(true))
                                .system_status_codes(|system| system.rel(true))
                        })
                })
        })
    .time_environment(time_environment)
    .build()
}
