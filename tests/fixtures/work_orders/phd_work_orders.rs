use chrono::TimeDelta;
use ordinator_orchestrator::Resources;
use ordinator_orchestrator::WorkOrderNumber;
use ordinator_scheduling_environment::work_order::WorkOrdersBuilder;
use ordinator_scheduling_environment::work_order::work_order_info::WorkOrderInfoDetail;
use ordinator_scheduling_environment::work_order::work_order_info::priority::Priority;
use ordinator_scheduling_environment::work_order::work_order_info::revision::Revision;
use ordinator_scheduling_environment::work_order::work_order_info::system_condition::SystemCondition;
use ordinator_scheduling_environment::work_order::work_order_info::work_order_text::WorkOrderText;
use ordinator_scheduling_environment::work_order::work_order_info::work_order_type::WorkOrderType;

/// These are the manually created `WorkOrder`s for the phd data set.
///
/// Numbering scheme is:
/// 1. 111199xxxx: normal filler work order, no binding status code, no vendor,
///    no shutdown.
/// 2. 222299xxxx: normal filler work order, no binding status code, no vendor,
///    with status code modifiers
/// 3. 333399xxxx: edgecase work orders, added to determine a particular aspect
pub fn phd_work_order_builder(wo_builder: WorkOrdersBuilder) -> WorkOrdersBuilder
{
    wo_builder
        .work_order_builder(WorkOrderNumber(1111990001), |wob| {
            wob.main_work_center(Resources::MtnMech)
                .operations_builder(10, Resources::MtnMech, |ob| {
                    ob.operation_info(|oib| oib.work_remaining(10.0).work(5.0).work_actual(5.0))
                        .operation_dates(|dates| {
                            dates
                                .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
                                .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
                        })
                        .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
                })
                .operations_builder(20, Resources::MtnElec, |ob| {
                    ob.operation_info(|oib| oib.work_remaining(10.0).work(5.0).work_actual(5.0))
                        .operation_dates(|dates| {
                            dates
                                .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
                                .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
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
                        .basic_start_from_ymd(2025, 1, 1)
                        .basic_finish_from_ymd(2025, 1, 1)
                        .earliest_allowed_start_from_ymd(2025, 1, 1)
                        .latest_allowed_finish_from_ymd(2025, 5, 1)
                })
                .work_order_analytic_builder(|woab| {
                    woab.user_status_codes(|user| user.smat(true).rel(true))
                        .system_status_codes(|system| system.rel(true))
                })
        })
        .work_order_builder(WorkOrderNumber(1111990002), |wob| {
            wob.main_work_center(Resources::MtnMech)
                .operations_builder(10, Resources::MtnMech, |ob| {
                    ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
                        .operation_dates(|dates| {
                            dates
                                .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
                                .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
                        })
                        .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
                })
                .operations_builder(20, Resources::MtnInst, |ob| {
                    ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
                        .operation_dates(|dates| {
                            dates
                                .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
                                .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
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
                        .basic_start_from_ymd(2025, 1, 1)
                        .basic_finish_from_ymd(2025, 1, 1)
                        .earliest_allowed_start_from_ymd(2025, 1, 1)
                        .latest_allowed_finish_from_ymd(2025, 5, 1)
                })
                .work_order_analytic_builder(|woab| {
                    woab.user_status_codes(|user| user.smat(true).rel(true))
                        .system_status_codes(|system| system.rel(true))
                })
        })
        .work_order_builder(WorkOrderNumber(1111990003), |wob| {
            wob.main_work_center(Resources::MtnMech)
                .operations_builder(10, Resources::MtnElec, |ob| {
                    ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
                        .operation_dates(|dates| {
                            dates
                                .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
                                .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
                        })
                        .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
                })
                .operations_builder(20, Resources::MtnMech, |ob| {
                    ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
                        .operation_dates(|dates| {
                            dates
                                .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
                                .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
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
                        .basic_start_from_ymd(2025, 1, 1)
                        .basic_finish_from_ymd(2025, 1, 1)
                        .earliest_allowed_start_from_ymd(2025, 1, 1)
                        .latest_allowed_finish_from_ymd(2025, 5, 1)
                })
                .work_order_analytic_builder(|woab| {
                    woab.user_status_codes(|user| user.smat(true).rel(true))
                        .system_status_codes(|system| system.rel(true))
                })
        })
        .work_order_builder(WorkOrderNumber(1111990004), |wob| {
            wob.main_work_center(Resources::MtnMech)
                .operations_builder(10, Resources::MtnMech, |ob| {
                    ob.operation_info(|oib| oib.work_remaining(10.0).work(5.0).work_actual(5.0))
                        .operation_dates(|dates| {
                            dates
                                .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
                                .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
                        })
                        .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
                })
                .operations_builder(20, Resources::MtnElec, |ob| {
                    ob.operation_info(|oib| oib.work_remaining(10.0).work(5.0).work_actual(5.0))
                        .operation_dates(|dates| {
                            dates
                                .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
                                .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
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
                        .basic_start_from_ymd(2025, 1, 1)
                        .basic_finish_from_ymd(2025, 1, 1)
                        .earliest_allowed_start_from_ymd(2025, 1, 1)
                        .latest_allowed_finish_from_ymd(2025, 5, 1)
                })
                .work_order_analytic_builder(|woab| {
                    woab.user_status_codes(|user| user.smat(true).rel(true))
                        .system_status_codes(|system| system.rel(true))
                })
        })
        .work_order_builder(WorkOrderNumber(1111990005), |wob| {
            wob.main_work_center(Resources::MtnMech)
                .operations_builder(10, Resources::MtnInst, |ob| {
                    ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
                        .operation_dates(|dates| {
                            dates
                                .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
                                .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
                        })
                        .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
                })
                .operations_builder(20, Resources::MtnElec, |ob| {
                    ob.operation_info(|oib| oib.work_remaining(4.0).work(5.0).work_actual(5.0))
                        .operation_dates(|dates| {
                            dates
                                .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
                                .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
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
                        .basic_start_from_ymd(2025, 1, 1)
                        .basic_finish_from_ymd(2025, 1, 1)
                        .earliest_allowed_start_from_ymd(2025, 1, 1)
                        .latest_allowed_finish_from_ymd(2025, 5, 1)
                })
                .work_order_analytic_builder(|woab| {
                    woab.user_status_codes(|user| user.smat(true).rel(true))
                        .system_status_codes(|system| system.rel(true))
                })
        })
        .work_order_builder(WorkOrderNumber(1111990006), |wob| {
            wob.main_work_center(Resources::MtnMech)
                .operations_builder(10, Resources::MtnMech, |ob| {
                    ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
                        .operation_dates(|dates| {
                            dates
                                .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
                                .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
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
                        .basic_start_from_ymd(2025, 1, 1)
                        .basic_finish_from_ymd(2025, 1, 1)
                        .earliest_allowed_start_from_ymd(2025, 1, 1)
                        .latest_allowed_finish_from_ymd(2025, 5, 1)
                })
                .work_order_analytic_builder(|woab| {
                    woab.user_status_codes(|user| user.smat(true).rel(true))
                        .system_status_codes(|system| system.rel(true))
                })
        })
}
