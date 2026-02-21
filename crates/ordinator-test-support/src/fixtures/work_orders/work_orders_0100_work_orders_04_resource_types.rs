use ordinator_scheduling_environment::work_order::WorkOrdersBuilder;

/// Builds manually created `WorkOrder`s for the phd data set.
///
/// Numbering scheme is:
/// 1. 111199xxxx: normal filler work order, no binding status code, no vendor,
///    no shutdown.
/// 2. 222299xxxx: normal filler work order, no binding status code, no vendor,
///    with status code modifiers
/// 3. 333399xxxx: edgecase work orders, added to determine a particular aspect
pub fn phd_work_order_builder(wo_builder: WorkOrdersBuilder) -> WorkOrdersBuilder
{
//     for work_order_data in work_order_datas {
//         wo_builder = wo_builder.work_order_builder(work_order_data.work_order_number, |wob| {
//             let mut wob = wob.main_work_center(Resources::MtnMech);
//             for opr in work_order_data.operations {
//                 wob = wob
//                     .operations_builder(opr.0, Resources::MtnMech, |ob| {
//                         ob.operation_info(|oib| {
//                             oib.work_remaining(opr.1).work(5.0).work_actual(5.0)
//                         })
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                     })
//                     .unwrap();
//             }
//             wob.work_order_info_builder(|woib| {
//                 woib.priority(Priority::new_int(3))
//                     // TODO: Only work order type is needed to fix this correctly in the code.
//                     .work_order_type(WorkOrderType::Wdf(Priority::new_int(3)))
//                     .revision(Revision::new("NOSD"))
//                     .work_order_text(WorkOrderText {
//                         order_system_status: Some("TEST".to_string()),
//                         order_user_status: Some("TEST".to_string()),
//                         order_description: "Basic Mechnical Job".to_string(),
//                         object_description: Some("TEST".to_string()),
//                         notes_1: Some("TEST".to_string()),
//                         notes_2: Some(1),
//                     })
//                     .functional_location_from_str("TEST/XX/XX/101")
//                     .system_condition(SystemCondition::A)
//                     // Requires comprehensive understanding of maintenance processes.
//                     .work_order_info_detail(WorkOrderInfoDetail {
//                         subnetwork: "123".to_string(),
//                         maintenance_plan: "PLAN TEST".to_string(),
//                         planner_group: "TEST_GROUP".to_string(),
//                         maintenance_plant: "TEST".to_string(),
//                         pm_collective: "TEST".to_string(),
//                         room: "TEST_ROOM".to_string(),
//                     })
//             })
//             .work_order_dates_builder(|wodb| {
//                 wodb.duration(TimeDelta::days(1))
//                     .basic_start_from_ymd(2025, 1, 1)
//                     .basic_finish_from_ymd(2025, 1, 1)
//                     .earliest_allowed_start_from_ymd(2025, 1, 1)
//                     .latest_allowed_finish_from_ymd(2025, 5, 1)
//             })
//             .work_order_analytic_builder(|woab| {
//                 woab.user_status_codes(|user| user.smat(true))
//                     .system_status_codes(|system| system.rel(true))
//             })
//         });
//     }

//     wo_builder
//         .work_order_builder(WorkOrderNumber(1111990000), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnMech, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::new_int(3))
//                         .work_order_type(WorkOrderType::Wdf(Priority::new_int(3)))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "Basic Mechnical Job".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990001), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnMech, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::new_int(3))
//                         .work_order_type(WorkOrderType::Wdf(Priority::new_int(3)))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "Basic Mechnical Job".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990002), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnMech, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::new_int(3))
//                         .work_order_type(WorkOrderType::Wdf(Priority::new_int(3)))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "Basic Mechnical Job".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990003), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnMech, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::new_int(3))
//                         .work_order_type(WorkOrderType::Wdf(Priority::new_int(3)))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "Basic Mechnical Job".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990004), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnMech, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::new_int(3))
//                         .work_order_type(WorkOrderType::Wdf(Priority::new_int(3)))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "Basic Mechnical Job".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990005), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnMech, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::new_int(3))
//                         .work_order_type(WorkOrderType::Wdf(Priority::new_int(3)))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "Basic Mechnical Job".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990006), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnMech, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::new_int(3))
//                         .work_order_type(WorkOrderType::Wdf(Priority::new_int(3)))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "Basic Mechnical Job".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990007), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnMech, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::new_int(3))
//                         .work_order_type(WorkOrderType::Wdf(Priority::new_int(3)))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "Basic Mechnical Job".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990008), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnMech, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::new_int(3))
//                         .work_order_type(WorkOrderType::Wdf(Priority::new_int(3)))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "Basic Mechnical Job".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990009), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnMech, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::new_int(3))
//                         .work_order_type(WorkOrderType::Wdf(Priority::new_int(3)))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "Basic Mechnical Job".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990010), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnElec, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::new_int(3))
//                         .work_order_type(WorkOrderType::Wdf(Priority::new_int(3)))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "BASIC ELEC JOB".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990011), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnElec, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::new_int(3))
//                         .work_order_type(WorkOrderType::Wdf(Priority::new_int(3)))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "BASIC ELEC JOB".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990012), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnElec, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::new_int(3))
//                         .work_order_type(WorkOrderType::Wdf(Priority::new_int(3)))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "BASIC ELEC JOB".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990013), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnElec, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::new_int(3))
//                         .work_order_type(WorkOrderType::Wdf(Priority::new_int(3)))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "BASIC ELEC JOB".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990014), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnElec, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::new_int(3))
//                         .work_order_type(WorkOrderType::Wdf(Priority::new_int(3)))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "BASIC ELEC JOB".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990015), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnElec, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::new_int(3))
//                         .work_order_type(WorkOrderType::Wdf(Priority::new_int(3)))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "BASIC ELEC JOB".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990016), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnElec, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::new_int(3))
//                         .work_order_type(WorkOrderType::Wdf(Priority::new_int(3)))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "BASIC ELEC JOB".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990017), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnElec, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::new_int(3))
//                         .work_order_type(WorkOrderType::Wdf(Priority::new_int(3)))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "BASIC ELEC JOB".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990018), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnElec, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::new_int(3))
//                         .work_order_type(WorkOrderType::Wdf(Priority::new_int(3)))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "BASIC ELEC JOB".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990019), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnElec, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::new_int(3))
//                         .work_order_type(WorkOrderType::Wdf(Priority::new_int(3)))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "BASIC ELEC JOB".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990020), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnInst, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::new_int(3))
//                         .work_order_type(WorkOrderType::Wdf(Priority::new_int(3)))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "Basic INST job".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990021), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnInst, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::new_int(3))
//                         .work_order_type(WorkOrderType::Wdf(Priority::new_int(3)))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "Basic INST job".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990022), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnInst, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::new_int(3))
//                         .work_order_type(WorkOrderType::Wdf(Priority::new_int(3)))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "Basic INST job".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990023), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnInst, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::new_int(3))
//                         .work_order_type(WorkOrderType::Wdf(Priority::new_int(3)))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "Basic INST job".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990024), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnInst, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::new_int(3))
//                         .work_order_type(WorkOrderType::Wdf(Priority::new_int(3)))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "Basic INST job".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990025), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnInst, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::new_int(3))
//                         .work_order_type(WorkOrderType::Wdf(Priority::new_int(3)))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "Basic INST job".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990026), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnInst, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::new_int(3))
//                         .work_order_type(WorkOrderType::Wdf(Priority::new_int(3)))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "Basic INST job".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990027), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnInst, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::new_int(3))
//                         .work_order_type(WorkOrderType::Wdf(Priority::new_int(3)))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "Basic INST job".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990028), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnInst, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::new_int(3))
//                         .work_order_type(WorkOrderType::Wdf(Priority::new_int(3)))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "Basic INST job".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990029), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnInst, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::new_int(3))
//                         .work_order_type(WorkOrderType::Wdf(Priority::new_int(3)))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "Basic INST job".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990030), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnMech, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .operations_builder(10, Resources::MtnElec, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::new_int(3))
//                         .work_order_type(WorkOrderType::Wdf(Priority::new_int(3)))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "MECH & ELEC JOB".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990031), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnMech, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 }).context("Could not build operation").unwrap()
//                 .operations_builder(10, Resources::MtnElec, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::new_int(3))
//                         .work_order_type(WorkOrderType::Wdf(Priority::new_int(3)))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "MECH & ELEC JOB".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990032), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnMech, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 }).context("Could not build operation").unwrap()
//                 .operations_builder(10, Resources::MtnElec, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::new_int(3))
//                         .work_order_type(WorkOrderType::Wdf(Priority::new_int(3)))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "MECH & ELEC JOB".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990033), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnMech, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 }).context("Could not build operation").unwrap()
//                 .operations_builder(10, Resources::MtnElec, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::new_int(3))
//                         .work_order_type(WorkOrderType::Wdf(Priority::new_int(3)))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "MECH & ELEC JOB".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990034), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnMech, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 }).context("Could not build operation").unwrap()
//                 .operations_builder(10, Resources::MtnElec, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::new_int(3))
//                         .work_order_type(WorkOrderType::Wdf(Priority::new_int(3)))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "MECH & ELEC JOB".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990035), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnMech, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 }).context("Could not build operation").unwrap()
//                 .operations_builder(10, Resources::MtnElec, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::new_int(3))
//                         .work_order_type(WorkOrderType::Wdf(Priority::new_int(3)))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "MECH & ELEC JOB".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990036), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnMech, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 }).context("Could not build operation").unwrap()
//                 .operations_builder(10, Resources::MtnElec, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::new_int(3))
//                         .work_order_type(WorkOrderType::Wdf(Priority::new_int(3)))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "MECH & ELEC JOB".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990037), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnMech, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 }).context("Could not build operation").unwrap()
//                 .operations_builder(10, Resources::MtnElec, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::new_int(3))
//                         .work_order_type(WorkOrderType::Wdf(Priority::new_int(3)))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "MECH & ELEC JOB".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990038), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnMech, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 }).context("Could not build operation").unwrap()
//                 .operations_builder(10, Resources::MtnElec, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::new_int(3))
//                         .work_order_type(WorkOrderType::Wdf(Priority::new_int(3)))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "MECH & ELEC JOB".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990039), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnMech, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 }).context("Could not build operation").unwrap()
//                 .operations_builder(10, Resources::MtnElec, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::new_int(3))
//                         .work_order_type(WorkOrderType::Wdf(Priority::new_int(3)))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "MECH & ELEC JOB".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990040), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnScaf, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 }).context("Could not build operation").unwrap()
//                 .operations_builder(10, Resources::MtnMech, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 }).context("Could not build operation").unwrap()
//                 .operations_builder(10, Resources::MtnScaf, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::new_int(3))
//                         .work_order_type(WorkOrderType::Wdf(Priority::new_int(3)))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "TEST".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990041), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnScaf, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 }).context("Could not build operation").unwrap()
//                 .operations_builder(10, Resources::MtnMech, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 }).context("Could not build operation").unwrap()
//                 .operations_builder(10, Resources::MtnScaf, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::new_int(3))
//                         .work_order_type(WorkOrderType::Wdf(Priority::new_int(3)))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "TEST".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990042), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnScaf, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 }).context("Could not build operation").unwrap()
//                 .operations_builder(10, Resources::MtnMech, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 }).context("Could not build operation").unwrap()
//                 .operations_builder(10, Resources::MtnScaf, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::new_int(3))
//                         .work_order_type(WorkOrderType::Wdf(Priority::new_int(3)))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "TEST".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990043), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnScaf, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 }).context("Could not build operation").unwrap()
//                 .operations_builder(10, Resources::MtnMech, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 }).context("Could not build operation").unwrap()
//                 .operations_builder(10, Resources::MtnScaf, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::new_int(3))
//                         .work_order_type(WorkOrderType::Wdf(Priority::new_int(3)))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "TEST".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990044), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnScaf, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 }).context("Could not build operation").unwrap()
//                 .operations_builder(10, Resources::MtnMech, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 }).context("Could not build operation").unwrap()
//                 .operations_builder(10, Resources::MtnScaf, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::new_int(3))
//                         .work_order_type(WorkOrderType::Wdf(Priority::new_int(3)))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "TEST".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990045), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnScaf, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 }).context("Could not build operation").unwrap()
//                 .operations_builder(10, Resources::MtnMech, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 }).context("Could not build operation").unwrap()
//                 .operations_builder(10, Resources::MtnScaf, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::new_int(3))
//                         .work_order_type(WorkOrderType::Wdf(Priority::new_int(3)))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "TEST".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990046), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnScaf, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 }).context("Could not build operation").unwrap()
//                 .operations_builder(10, Resources::MtnMech, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 }).context("Could not build operation").unwrap()
//                 .operations_builder(10, Resources::MtnScaf, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::new_int(3))
//                         .work_order_type(WorkOrderType::Wdf(Priority::new_int(3)))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "TEST".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990047), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnScaf, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 }).context("Could not build operation").unwrap()
//                 .operations_builder(10, Resources::MtnMech, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 }).context("Could not build operation").unwrap()
//                 .operations_builder(10, Resources::MtnScaf, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::new_int(3))
//                         .work_order_type(WorkOrderType::Wdf(Priority::new_int(3)))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "TEST".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990048), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnScaf, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 }).context("Could not build operation").unwrap()
//                 .operations_builder(10, Resources::MtnMech, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 }).context("Could not build operation").unwrap()
//                 .operations_builder(10, Resources::MtnScaf, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::new_int(3))
//                         .work_order_type(WorkOrderType::Wdf(Priority::new_int(3)))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "TEST".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990049), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnScaf, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 }).context("Could not build operation").unwrap()
//                 .operations_builder(10, Resources::MtnMech, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 }).context("Could not build operation").unwrap()
//                 .operations_builder(10, Resources::MtnScaf, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::new_int(3))
//                         .work_order_type(WorkOrderType::Wdf(Priority::new_int(3)))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "TEST".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990050), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnMech, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::new_int(1))
//                         .work_order_type(WorkOrderType::Wdf(Priority::new_int(1)))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "TEST".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990051), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnMech, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::new_int(1))
//                         .work_order_type(WorkOrderType::Wdf(Priority::new_int(1)))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "TEST".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990052), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnMech, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::new_int(1))
//                         .work_order_type(WorkOrderType::Wdf(Priority::new_int(1)))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "TEST".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990053), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnMech, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::new_int(1))
//                         .work_order_type(WorkOrderType::Wdf(Priority::new_int(1)))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "TEST".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990054), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnMech, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::new_int(1))
//                         .work_order_type(WorkOrderType::Wdf(Priority::new_int(1)))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "TEST".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990055), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnMech, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::new_int(1))
//                         .work_order_type(WorkOrderType::Wdf(Priority::new_int(1)))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "TEST".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990056), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnMech, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::new_int(1))
//                         .work_order_type(WorkOrderType::Wdf(Priority::new_int(1)))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "TEST".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990057), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnMech, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::new_int(1))
//                         .work_order_type(WorkOrderType::Wdf(Priority::new_int(1)))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "TEST".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990058), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnMech, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::new_int(1))
//                         .work_order_type(WorkOrderType::Wdf(Priority::new_int(1)))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "TEST".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990059), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnMech, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::new_int(1))
//                         .work_order_type(WorkOrderType::Wdf(Priority::new_int(1)))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "TEST".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990060), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnMech, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(1.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::Char('B'))
//                         .work_order_type(WorkOrderType::Wpm(Priority::Char('B')))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "TEST".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990061), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnMech, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(2.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::Char('B'))
//                         .work_order_type(WorkOrderType::Wpm(Priority::Char('B')))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "TEST".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990062), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnMech, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(3.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::Char('B'))
//                         .work_order_type(WorkOrderType::Wpm(Priority::Char('B')))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "TEST".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990063), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnMech, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(4.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::Char('B'))
//                         .work_order_type(WorkOrderType::Wpm(Priority::Char('B')))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "TEST".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990064), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnMech, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::Char('B'))
//                         .work_order_type(WorkOrderType::Wpm(Priority::Char('B')))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "TEST".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990065), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnMech, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(6.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::Char('B'))
//                         .work_order_type(WorkOrderType::Wpm(Priority::Char('B')))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "TEST".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990066), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnMech, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(7.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::Char('B'))
//                         .work_order_type(WorkOrderType::Wpm(Priority::Char('B')))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "TEST".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990067), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnMech, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(8.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::Char('B'))
//                         .work_order_type(WorkOrderType::Wpm(Priority::Char('B')))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "TEST".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990068), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnMech, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(9.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::Char('B'))
//                         .work_order_type(WorkOrderType::Wpm(Priority::Char('B')))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "TEST".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990069), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnMech, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(10.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::Char('B'))
//                         .work_order_type(WorkOrderType::Wpm(Priority::Char('B')))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "TEST".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990070), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnElec, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(1.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::Char('B'))
//                         .work_order_type(WorkOrderType::Wpm(Priority::Char('B')))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "TEST".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990071), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnElec, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(2.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::Char('B'))
//                         .work_order_type(WorkOrderType::Wpm(Priority::Char('B')))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "TEST".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990072), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnElec, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(3.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::Char('B'))
//                         .work_order_type(WorkOrderType::Wpm(Priority::Char('B')))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "TEST".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990073), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnElec, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(4.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::Char('B'))
//                         .work_order_type(WorkOrderType::Wpm(Priority::Char('B')))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "TEST".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990074), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnElec, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::Char('B'))
//                         .work_order_type(WorkOrderType::Wpm(Priority::Char('B')))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "TEST".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990075), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnElec, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(6.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::Char('B'))
//                         .work_order_type(WorkOrderType::Wpm(Priority::Char('B')))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "TEST".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990076), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnElec, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(7.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::Char('B'))
//                         .work_order_type(WorkOrderType::Wpm(Priority::Char('B')))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "TEST".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990077), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnElec, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(8.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::Char('B'))
//                         .work_order_type(WorkOrderType::Wpm(Priority::Char('B')))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "TEST".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990078), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnElec, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(9.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::Char('B'))
//                         .work_order_type(WorkOrderType::Wpm(Priority::Char('B')))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "TEST".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990079), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnElec, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(10.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::Char('B'))
//                         .work_order_type(WorkOrderType::Wpm(Priority::Char('B')))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "TEST".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990080), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnMech, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(1.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::Char('D'))
//                         .work_order_type(WorkOrderType::Wpm(Priority::Char('D')))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "TEST".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990081), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnMech, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(2.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::Char('D'))
//                         .work_order_type(WorkOrderType::Wpm(Priority::Char('D')))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "TEST".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990082), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnMech, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(3.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::Char('D'))
//                         .work_order_type(WorkOrderType::Wpm(Priority::Char('D')))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "TEST".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990083), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnMech, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(4.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::Char('D'))
//                         .work_order_type(WorkOrderType::Wpm(Priority::Char('D')))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "TEST".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990084), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnMech, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::Char('D'))
//                         .work_order_type(WorkOrderType::Wpm(Priority::Char('D')))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "TEST".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990085), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnMech, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(6.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::Char('D'))
//                         .work_order_type(WorkOrderType::Wpm(Priority::Char('D')))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "TEST".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990086), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnMech, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(7.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::Char('D'))
//                         .work_order_type(WorkOrderType::Wpm(Priority::Char('D')))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "TEST".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990087), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnMech, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(8.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::Char('D'))
//                         .work_order_type(WorkOrderType::Wpm(Priority::Char('D')))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "TEST".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990088), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnMech, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(9.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::Char('D'))
//                         .work_order_type(WorkOrderType::Wpm(Priority::Char('D')))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "TEST".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990089), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnMech, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(10.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::Char('D'))
//                         .work_order_type(WorkOrderType::Wpm(Priority::Char('D')))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "TEST".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990090), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnElec, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(1.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::new_int(1))
//                         .work_order_type(WorkOrderType::Wdf(Priority::new_int(1)))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "TEST".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990091), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnElec, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(2.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::new_int(1))
//                         .work_order_type(WorkOrderType::Wdf(Priority::new_int(1)))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "TEST".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990092), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnElec, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(3.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::new_int(1))
//                         .work_order_type(WorkOrderType::Wdf(Priority::new_int(1)))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "TEST".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990093), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnElec, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(4.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::new_int(1))
//                         .work_order_type(WorkOrderType::Wdf(Priority::new_int(1)))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "TEST".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990094), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnElec, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::new_int(1))
//                         .work_order_type(WorkOrderType::Wdf(Priority::new_int(1)))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "TEST".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990095), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnElec, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(6.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::new_int(1))
//                         .work_order_type(WorkOrderType::Wdf(Priority::new_int(1)))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "TEST".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990096), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnElec, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(7.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::new_int(1))
//                         .work_order_type(WorkOrderType::Wdf(Priority::new_int(1)))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "TEST".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990097), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnElec, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(8.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::new_int(1))
//                         .work_order_type(WorkOrderType::Wdf(Priority::new_int(1)))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "TEST".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990098), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnElec, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(9.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::new_int(1))
//                         .work_order_type(WorkOrderType::Wdf(Priority::new_int(1)))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "TEST".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
//         .work_order_builder(WorkOrderNumber(1111990099), |wob| {
//             wob.main_work_center(Resources::MtnMech)
//                 .operations_builder(10, Resources::MtnElec, |ob| {
//                     ob.operation_info(|oib| oib.work_remaining(10.0).work(5.0).work_actual(5.0))
//                         .operation_dates(|dates| {
//                             dates
//                                 .earliest_start_from_ymd_hms(2025, 1, 1, 7, 0, 0)
//                                 .earliest_finish_from_ymd_hms(2025, 1, 2, 7, 0, 0)
//                         })
//                         .operation_analytic(|oab| oab.duration(1.0).preparation_time(1.0))
//                 })
//                 .with_context(|| format!("Operation could not be added to WorkOrder"))
//                 .unwrap()
//                 .work_order_info_builder(|woib| {
//                     woib.priority(Priority::new_int(1))
//                         .work_order_type(WorkOrderType::Wdf(Priority::new_int(1)))
//                         .revision(Revision::new("NOSD"))
//                         .work_order_text(WorkOrderText {
//                             order_system_status: Some("TEST".to_string()),
//                             order_user_status: Some("TEST".to_string()),
//                             order_description: "TEST".to_string(),
//                             object_description: Some("TEST".to_string()),
//                             notes_1: Some("TEST".to_string()),
//                             notes_2: Some(1),
//                         })
//                         .functional_location_from_str("TEST/XX/XX/101")
//                         .system_condition(SystemCondition::A)
//                         // Requires comprehensive understanding of maintenance processes.
//                         .work_order_info_detail(WorkOrderInfoDetail {
//                             subnetwork: "123".to_string(),
//                             maintenance_plan: "PLAN TEST".to_string(),
//                             planner_group: "TEST_GROUP".to_string(),
//                             maintenance_plant: "TEST".to_string(),
//                             pm_collective: "TEST".to_string(),
//                             room: "TEST_ROOM".to_string(),
//                         })
//                 })
//                 .work_order_dates_builder(|wodb| {
//                     wodb.duration(TimeDelta::days(1))
//                         .basic_start_from_ymd(2025, 1, 1)
//                         .basic_finish_from_ymd(2025, 1, 1)
//                         .earliest_allowed_start_from_ymd(2025, 1, 1)
//                         .latest_allowed_finish_from_ymd(2025, 5, 1)
//                 })
//                 .work_order_analytic_builder(|woab| {
//                     woab.user_status_codes(|user| user.smat(true))
//                         .system_status_codes(|system| system.rel(true))
//                 })
//         })
    wo_builder
}

// TODO: Start here - add more WorkOrder instances to the code
// TODO: Plot the results
// TODO: Add special cases for testing EASD and material respect
// TODO: Implement constraint satisfaction to the problem and UI
