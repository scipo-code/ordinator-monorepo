use std::panic::Location;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use chrono::DateTime;
use chrono::NaiveDate;
use chrono::NaiveTime;
use chrono::TimeDelta;
use chrono::TimeZone;
use chrono::Utc;
use ordinator_orchestrator::Asset;
use ordinator_orchestrator::Availability;
use ordinator_orchestrator::WorkOrderNumber;
use ordinator_scheduling_environment::SchedulingEnvironment;
use ordinator_scheduling_environment::materials::MaterialRepo;
use ordinator_scheduling_environment::materials::MaterialToPeriod;
use ordinator_scheduling_environment::time_environment::TimeInterval;
use ordinator_scheduling_environment::time_environment::create_time_environment;
use ordinator_scheduling_environment::work_order::WorkOrderPolicies;
use ordinator_scheduling_environment::work_order::work_order_info::WorkOrderInfoDetail;
use ordinator_scheduling_environment::work_order::work_order_info::priority::Priority;
use ordinator_scheduling_environment::work_order::work_order_info::revision::Revision;
use ordinator_scheduling_environment::work_order::work_order_info::system_condition::SystemCondition;
use ordinator_scheduling_environment::work_order::work_order_info::work_order_text::WorkOrderText;
use ordinator_scheduling_environment::work_order::work_order_info::work_order_type::WorkOrderType;
use ordinator_scheduling_environment::worker_environment::ActorSpecificationBuilder;
use ordinator_scheduling_environment::worker_environment::TimeInput;
use ordinator_scheduling_environment::worker_environment::resources::Resources;

pub fn load_scheduling_environment() -> Arc<std::sync::Mutex<SchedulingEnvironment>>
{
    let asset = Asset::Test;
    let asset_string = asset.to_string().to_lowercase();

    // ISSUE [ ] - this should be removed to and replaced with builders.
    let path_to_workers = PathBuf::from(format!(
        "temp_scheduling_environment_database/actor_specifications/actor_specification_{asset_string}.toml",
    ));
    let path_to_workers = Path::new(env!("CARGO_MANIFEST_DIR")).join(path_to_workers);

    println!("{}\n{}", path_to_workers.display(), Location::caller());

    let time_input = TimeInput {
        number_of_periods: 5,
        number_of_days: 42,
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

fn work_order_builder(
    wo_builder: ordinator_scheduling_environment::work_order::WorkOrdersBuilder,
) -> ordinator_scheduling_environment::work_order::WorkOrdersBuilder
{
    wo_builder
        .work_order_builder(WorkOrderNumber(2100001234), |wob| {
            wob.main_work_center(Resources::MtnMech)
                .operations_builder(10, Resources::MtnMech, |ob| {
                    ob.operation_info(|oib| oib.work_remaining(10.0).work(5.0).work_actual(5.0))
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
                .operations_builder(20, Resources::MtnElec, |ob| {
                    ob.operation_info(|oib| oib.work_remaining(10.0).work(5.0).work_actual(5.0))
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
                        .basic_start_date(NaiveDate::from_ymd_opt(2025, 1, 1).expect(
                            "This date is required for constructing a WorkOrderDates object",
                        ))
                        .basic_finish_date(NaiveDate::from_ymd_opt(2025, 1, 1).expect(
                            "This date is required for constructing a WorkOrderDates object",
                        ))
                        .earliest_allowed_start_date(NaiveDate::from_ymd_opt(2025, 1, 1).expect(
                            "This date is required for constructing a WorkOrderDates object",
                        ))
                        .latest_allowed_finish_date(NaiveDate::from_ymd_opt(2025, 5, 1).expect(
                            "This date is required for constructing a WorkOrderDates object",
                        ))
                })
                .work_order_analytic_builder(|woab| {
                    woab.user_status_codes(|user| user.smat(true).rel(true))
                        .system_status_codes(|system| system.rel(true))
                })
        })
        .work_order_builder(WorkOrderNumber(2200001234), |wob| {
            wob.main_work_center(Resources::MtnMech)
                .operations_builder(10, Resources::MtnMech, |ob| {
                    ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
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
                .operations_builder(20, Resources::MtnInst, |ob| {
                    ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
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
                        .basic_start_date(NaiveDate::from_ymd_opt(2025, 1, 1).expect(
                            "This date is required for constructing a WorkOrderDates object",
                        ))
                        .basic_finish_date(NaiveDate::from_ymd_opt(2025, 1, 1).expect(
                            "This date is required for constructing a WorkOrderDates object",
                        ))
                        .earliest_allowed_start_date(NaiveDate::from_ymd_opt(2025, 1, 1).expect(
                            "This date is required for constructing a WorkOrderDates object",
                        ))
                        .latest_allowed_finish_date(NaiveDate::from_ymd_opt(2025, 5, 1).expect(
                            "This date is required for constructing a WorkOrderDates object",
                        ))
                })
                .work_order_analytic_builder(|woab| {
                    woab.user_status_codes(|user| user.smat(true).rel(true))
                        .system_status_codes(|system| system.rel(true))
                })
        })
        .work_order_builder(WorkOrderNumber(2300001234), |wob| {
            wob.main_work_center(Resources::MtnMech)
                .operations_builder(10, Resources::MtnElec, |ob| {
                    ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
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
                .operations_builder(20, Resources::MtnMech, |ob| {
                    ob.operation_info(|oib| oib.work_remaining(5.0).work(5.0).work_actual(5.0))
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
                        .basic_start_date(NaiveDate::from_ymd_opt(2025, 1, 1).expect(
                            "This date is required for constructing a WorkOrderDates object",
                        ))
                        .basic_finish_date(NaiveDate::from_ymd_opt(2025, 1, 1).expect(
                            "This date is required for constructing a WorkOrderDates object",
                        ))
                        .earliest_allowed_start_date(NaiveDate::from_ymd_opt(2025, 1, 1).expect(
                            "This date is required for constructing a WorkOrderDates object",
                        ))
                        .latest_allowed_finish_date(NaiveDate::from_ymd_opt(2025, 5, 1).expect(
                            "This date is required for constructing a WorkOrderDates object",
                        ))
                })
                .work_order_analytic_builder(|woab| {
                    woab.user_status_codes(|user| user.smat(true).rel(true))
                        .system_status_codes(|system| system.rel(true))
                })
        })
}

fn worker_builder(actor_builder: ActorSpecificationBuilder) -> ActorSpecificationBuilder
{
    actor_builder
        .strategic(|strategic| {
            strategic
                .id("TEST_STRATEGIC")
                .number_of_strategic_periods(52)
                .strategic_options(|f| {
                    f.number_of_removed_work_orders(5)
                        .urgency_weight(1000)
                        .resource_penalty_weight(1_000_000)
                        .clustering_weight(1000)
                })
        })
        .tactical(|tactical| {
            tactical
                .id("TEST_TACTICAL")
                .number_of_tactical_days(120)
                .tactical_options(|f| {
                    f.number_of_removed_work_orders(20)
                        .urgency(1000)
                        .resource_penalty(100000)
                })
        })
        .supervisors(|supervisor| {
            supervisor.supervisor(|f| {
                f.id("TEST_SUPERVISOR")
                    .number_of_supervisor_periods(2)
                    .supervisor_options(|f| f.number_of_unassigned_work_orders(10))
            })
        })
        .operational(|operational_builder| {
            operational_builder
                .operational("TEST_OP-001-01", |operational_actor| {
                    operational_actor
                        .hours_per_day(6.0)
                        .operational_configuration(|config| {
                            config
                                .add_availability(
                                    Availability::new(
                                        DateTime::parse_from_rfc3339("2025-01-13T07:00:00Z")
                                            .unwrap()
                                            .to_utc(),
                                        DateTime::parse_from_rfc3339("2025-01-27T15:00:00Z")
                                            .unwrap()
                                            .to_utc(),
                                        vec![Asset::Test],
                                    )
                                    .unwrap(),
                                )
                                .add_resource(Resources::MtnMech)
                                .break_interval(
                                    TimeInterval::new(
                                        NaiveTime::from_hms_opt(11, 0, 0).unwrap(),
                                        NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
                                    )
                                    .unwrap(),
                                )
                                .off_shift_interval(
                                    TimeInterval::new(
                                        NaiveTime::from_hms_opt(19, 0, 0).unwrap(),
                                        NaiveTime::from_hms_opt(7, 0, 0).unwrap(),
                                    )
                                    .unwrap(),
                                )
                                .toolbox_interval(
                                    TimeInterval::new(
                                        NaiveTime::from_hms_opt(7, 0, 0).unwrap(),
                                        NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
                                    )
                                    .unwrap(),
                                )
                        })
                        .operational_options(|options| options.number_of_removed_activities(10))
                })
                .operational("TEST_OP-001-02", |operational_actor_2| {
                    operational_actor_2
                        .hours_per_day(6.0)
                        .operational_configuration(|config| {
                            config
                                .add_availability(
                                    Availability::new(
                                        DateTime::parse_from_rfc3339("2025-01-13T07:00:00Z")
                                            .unwrap()
                                            .to_utc(),
                                        DateTime::parse_from_rfc3339("2025-01-27T15:00:00Z")
                                            .unwrap()
                                            .to_utc(),
                                        vec![Asset::Test],
                                    )
                                    .unwrap(),
                                )
                                .add_resource(Resources::MtnElec)
                                // THIS IS WHAT YOU SHOULD NOT DO! Each you are spilling out 3 types
                                // that should be encapsulated.
                                .break_interval(
                                    TimeInterval::new(
                                        NaiveTime::from_hms_opt(11, 0, 0).unwrap(),
                                        NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
                                    )
                                    .unwrap(),
                                )
                                .off_shift_interval(
                                    TimeInterval::new(
                                        NaiveTime::from_hms_opt(19, 0, 0).unwrap(),
                                        NaiveTime::from_hms_opt(7, 0, 0).unwrap(),
                                    )
                                    .unwrap(),
                                )
                                .toolbox_interval(
                                    TimeInterval::new(
                                        NaiveTime::from_hms_opt(7, 0, 0).unwrap(),
                                        NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
                                    )
                                    .unwrap(),
                                )
                        })
                        .operational_options(|options| options.number_of_removed_activities(10))
                })
                .operational("TEST_OP-002-01", |operational_actor_2| {
                    operational_actor_2
                        .hours_per_day(6.0)
                        .operational_configuration(|config| {
                            config
                                .add_availability(
                                    Availability::new(
                                        DateTime::parse_from_rfc3339("2025-01-13T07:00:00Z")
                                            .unwrap()
                                            .to_utc(),
                                        DateTime::parse_from_rfc3339("2025-01-27T15:00:00Z")
                                            .unwrap()
                                            .to_utc(),
                                        vec![Asset::Test],
                                    )
                                    .unwrap(),
                                )
                                .add_resource(Resources::MtnInst)
                                .break_interval(
                                    TimeInterval::new(
                                        NaiveTime::from_hms_opt(11, 0, 0).unwrap(),
                                        NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
                                    )
                                    .unwrap(),
                                )
                                .off_shift_interval(
                                    TimeInterval::new(
                                        NaiveTime::from_hms_opt(19, 0, 0).unwrap(),
                                        NaiveTime::from_hms_opt(7, 0, 0).unwrap(),
                                    )
                                    .unwrap(),
                                )
                                .toolbox_interval(
                                    TimeInterval::new(
                                        NaiveTime::from_hms_opt(7, 0, 0).unwrap(),
                                        NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
                                    )
                                    .unwrap(),
                                )
                        })
                        .operational_options(|options| options.number_of_removed_activities(10))
                })
        })
}
