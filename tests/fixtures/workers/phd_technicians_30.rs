use ordinator_orchestrator::Asset;
use ordinator_orchestrator::Availability;
use ordinator_orchestrator::Resources;
use ordinator_scheduling_environment::time_environment::TimeInterval;
use ordinator_scheduling_environment::worker_environment::ActorSpecificationBuilder;

/// * 4 Mech     (DONE)
/// * 4 Elec     (DONE)
/// * 2 Inst     (DONE)
/// * 2 Lagg     (DONE)
/// * 2 Tele     (DONE)
/// * 2 Rope     (DONE)
/// * 2 Cran     (DONE)
/// * 3 Scaf     (DONE)
/// * 3 Rigg     (DONE)
/// * 3 Rous     (DONE)
/// * 3 Prodtech (DONE)
/// *
///
/// Total: 30
pub fn phd_workers_builder(actor_builder: ActorSpecificationBuilder) -> ActorSpecificationBuilder
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
                .operational("TEST_OP-MtnMech-00", |operational_actor| {
                    operational_actor
                        .hours_per_day(6.0)
                        .operational_configuration(|config| {
                            config
                                .add_availability(
                                    Availability::from_rfc3339_strings(
                                        "2025-01-13T07:00:00Z",
                                        "2025-01-27T15:00:00Z",
                                        vec![Asset::Test],
                                    )
                                    .unwrap(),
                                )
                                .add_resource(Resources::MtnMech)
                                .break_interval(TimeInterval::from_hms(11, 0, 0, 12, 0, 0).unwrap())
                                .off_shift_interval(
                                    TimeInterval::from_hms(19, 0, 0, 7, 0, 0).unwrap(),
                                )
                                .toolbox_interval(TimeInterval::from_hms(7, 0, 0, 8, 0, 0).unwrap())
                        })
                        .operational_options(|options| options.number_of_removed_activities(10))
                })
                .operational("TEST_OP-MtnMech-01", |operational_actor| {
                    operational_actor
                        .hours_per_day(6.0)
                        .operational_configuration(|config| {
                            config
                                .add_availability(
                                    Availability::from_rfc3339_strings(
                                        "2025-01-13T07:00:00Z",
                                        "2025-01-27T15:00:00Z",
                                        vec![Asset::Test],
                                    )
                                    .unwrap(),
                                )
                                .add_resource(Resources::MtnMech)
                                .break_interval(TimeInterval::from_hms(11, 0, 0, 12, 0, 0).unwrap())
                                .off_shift_interval(
                                    TimeInterval::from_hms(19, 0, 0, 7, 0, 0).unwrap(),
                                )
                                .toolbox_interval(TimeInterval::from_hms(7, 0, 0, 8, 0, 0).unwrap())
                        })
                        .operational_options(|options| options.number_of_removed_activities(10))
                })
                .operational("TEST_OP-MtnMech-02", |operational_actor| {
                    operational_actor
                        .hours_per_day(6.0)
                        .operational_configuration(|config| {
                            config
                                .add_availability(
                                    Availability::from_rfc3339_strings(
                                        "2025-01-13T07:00:00Z",
                                        "2025-01-27T15:00:00Z",
                                        vec![Asset::Test],
                                    )
                                    .unwrap(),
                                )
                                .add_resource(Resources::MtnMech)
                                .break_interval(TimeInterval::from_hms(11, 0, 0, 12, 0, 0).unwrap())
                                .off_shift_interval(
                                    TimeInterval::from_hms(19, 0, 0, 7, 0, 0).unwrap(),
                                )
                                .toolbox_interval(TimeInterval::from_hms(7, 0, 0, 8, 0, 0).unwrap())
                        })
                        .operational_options(|options| options.number_of_removed_activities(10))
                })
                .operational("TEST_OP-MtnMech-03", |operational_actor| {
                    operational_actor
                        .hours_per_day(6.0)
                        .operational_configuration(|config| {
                            config
                                .add_availability(
                                    Availability::from_rfc3339_strings(
                                        "2025-01-13T07:00:00Z",
                                        "2025-01-27T15:00:00Z",
                                        vec![Asset::Test],
                                    )
                                    .unwrap(),
                                )
                                .add_resource(Resources::MtnMech)
                                .break_interval(TimeInterval::from_hms(11, 0, 0, 12, 0, 0).unwrap())
                                .off_shift_interval(
                                    TimeInterval::from_hms(19, 0, 0, 7, 0, 0).unwrap(),
                                )
                                .toolbox_interval(TimeInterval::from_hms(7, 0, 0, 8, 0, 0).unwrap())
                        })
                        .operational_options(|options| options.number_of_removed_activities(10))
                })
                .operational("TEST_OP-MtnElec-00", |operational_actor| {
                    operational_actor
                        .hours_per_day(6.0)
                        .operational_configuration(|config| {
                            config
                                .add_availability(
                                    Availability::from_rfc3339_strings(
                                        "2025-01-13T07:00:00Z",
                                        "2025-01-27T15:00:00Z",
                                        vec![Asset::Test],
                                    )
                                    .unwrap(),
                                )
                                .add_resource(Resources::MtnElec)
                                .break_interval(TimeInterval::from_hms(11, 0, 0, 12, 0, 0).unwrap())
                                .off_shift_interval(
                                    TimeInterval::from_hms(19, 0, 0, 7, 0, 0).unwrap(),
                                )
                                .toolbox_interval(TimeInterval::from_hms(7, 0, 0, 8, 0, 0).unwrap())
                        })
                        .operational_options(|options| options.number_of_removed_activities(10))
                })
                .operational("TEST_OP-MtnElec-01", |operational_actor| {
                    operational_actor
                        .hours_per_day(6.0)
                        .operational_configuration(|config| {
                            config
                                .add_availability(
                                    Availability::from_rfc3339_strings(
                                        "2025-01-13T07:00:00Z",
                                        "2025-01-27T15:00:00Z",
                                        vec![Asset::Test],
                                    )
                                    .unwrap(),
                                )
                                .add_resource(Resources::MtnElec)
                                .break_interval(TimeInterval::from_hms(11, 0, 0, 12, 0, 0).unwrap())
                                .off_shift_interval(
                                    TimeInterval::from_hms(19, 0, 0, 7, 0, 0).unwrap(),
                                )
                                .toolbox_interval(TimeInterval::from_hms(7, 0, 0, 8, 0, 0).unwrap())
                        })
                        .operational_options(|options| options.number_of_removed_activities(10))
                })
                .operational("TEST_OP-MtnElec-02", |operational_actor| {
                    operational_actor
                        .hours_per_day(6.0)
                        .operational_configuration(|config| {
                            config
                                .add_availability(
                                    Availability::from_rfc3339_strings(
                                        "2025-01-13T07:00:00Z",
                                        "2025-01-27T15:00:00Z",
                                        vec![Asset::Test],
                                    )
                                    .unwrap(),
                                )
                                .add_resource(Resources::MtnElec)
                                .break_interval(TimeInterval::from_hms(11, 0, 0, 12, 0, 0).unwrap())
                                .off_shift_interval(
                                    TimeInterval::from_hms(19, 0, 0, 7, 0, 0).unwrap(),
                                )
                                .toolbox_interval(TimeInterval::from_hms(7, 0, 0, 8, 0, 0).unwrap())
                        })
                        .operational_options(|options| options.number_of_removed_activities(10))
                })
                .operational("TEST_OP-MtnElec-03", |operational_actor| {
                    operational_actor
                        .hours_per_day(6.0)
                        .operational_configuration(|config| {
                            config
                                .add_availability(
                                    Availability::from_rfc3339_strings(
                                        "2025-01-13T07:00:00Z",
                                        "2025-01-27T15:00:00Z",
                                        vec![Asset::Test],
                                    )
                                    .unwrap(),
                                )
                                .add_resource(Resources::MtnElec)
                                .break_interval(TimeInterval::from_hms(11, 0, 0, 12, 0, 0).unwrap())
                                .off_shift_interval(
                                    TimeInterval::from_hms(19, 0, 0, 7, 0, 0).unwrap(),
                                )
                                .toolbox_interval(TimeInterval::from_hms(7, 0, 0, 8, 0, 0).unwrap())
                        })
                        .operational_options(|options| options.number_of_removed_activities(10))
                })
                .operational("TEST_OP-MtnInst-00", |operational_actor| {
                    operational_actor
                        .hours_per_day(6.0)
                        .operational_configuration(|config| {
                            config
                                .add_availability(
                                    Availability::from_rfc3339_strings(
                                        "2025-01-13T07:00:00Z",
                                        "2025-01-27T15:00:00Z",
                                        vec![Asset::Test],
                                    )
                                    .unwrap(),
                                )
                                .add_resource(Resources::MtnInst)
                                .break_interval(TimeInterval::from_hms(11, 0, 0, 12, 0, 0).unwrap())
                                .off_shift_interval(
                                    TimeInterval::from_hms(19, 0, 0, 7, 0, 0).unwrap(),
                                )
                                .toolbox_interval(TimeInterval::from_hms(7, 0, 0, 8, 0, 0).unwrap())
                        })
                        .operational_options(|options| options.number_of_removed_activities(10))
                })
                .operational("TEST_OP-MtnInst-01", |operational_actor| {
                    operational_actor
                        .hours_per_day(6.0)
                        .operational_configuration(|config| {
                            config
                                .add_availability(
                                    Availability::from_rfc3339_strings(
                                        "2025-01-13T07:00:00Z",
                                        "2025-01-27T15:00:00Z",
                                        vec![Asset::Test],
                                    )
                                    .unwrap(),
                                )
                                .add_resource(Resources::MtnInst)
                                .break_interval(TimeInterval::from_hms(11, 0, 0, 12, 0, 0).unwrap())
                                .off_shift_interval(
                                    TimeInterval::from_hms(19, 0, 0, 7, 0, 0).unwrap(),
                                )
                                .toolbox_interval(TimeInterval::from_hms(7, 0, 0, 8, 0, 0).unwrap())
                        })
                        .operational_options(|options| options.number_of_removed_activities(10))
                })
                .operational("TEST_OP-MtnLagg-00", |operational_actor| {
                    operational_actor
                        .hours_per_day(6.0)
                        .operational_configuration(|config| {
                            config
                                .add_availability(
                                    Availability::from_rfc3339_strings(
                                        "2025-01-13T07:00:00Z",
                                        "2025-01-27T15:00:00Z",
                                        vec![Asset::Test],
                                    )
                                    .unwrap(),
                                )
                                .add_resource(Resources::MtnLagg)
                                .break_interval(TimeInterval::from_hms(11, 0, 0, 12, 0, 0).unwrap())
                                .off_shift_interval(
                                    TimeInterval::from_hms(19, 0, 0, 7, 0, 0).unwrap(),
                                )
                                .toolbox_interval(TimeInterval::from_hms(7, 0, 0, 8, 0, 0).unwrap())
                        })
                        .operational_options(|options| options.number_of_removed_activities(10))
                })
                .operational("TEST_OP-MtnLagg-01", |operational_actor| {
                    operational_actor
                        .hours_per_day(6.0)
                        .operational_configuration(|config| {
                            config
                                .add_availability(
                                    Availability::from_rfc3339_strings(
                                        "2025-01-13T07:00:00Z",
                                        "2025-01-27T15:00:00Z",
                                        vec![Asset::Test],
                                    )
                                    .unwrap(),
                                )
                                .add_resource(Resources::MtnLagg)
                                .break_interval(TimeInterval::from_hms(11, 0, 0, 12, 0, 0).unwrap())
                                .off_shift_interval(
                                    TimeInterval::from_hms(19, 0, 0, 7, 0, 0).unwrap(),
                                )
                                .toolbox_interval(TimeInterval::from_hms(7, 0, 0, 8, 0, 0).unwrap())
                        })
                        .operational_options(|options| options.number_of_removed_activities(10))
                })
                .operational("TEST_OP-MtnTele-00", |operational_actor| {
                    operational_actor
                        .hours_per_day(6.0)
                        .operational_configuration(|config| {
                            config
                                .add_availability(
                                    Availability::from_rfc3339_strings(
                                        "2025-01-13T07:00:00Z",
                                        "2025-01-27T15:00:00Z",
                                        vec![Asset::Test],
                                    )
                                    .unwrap(),
                                )
                                .add_resource(Resources::MtnTele)
                                .break_interval(TimeInterval::from_hms(11, 0, 0, 12, 0, 0).unwrap())
                                .off_shift_interval(
                                    TimeInterval::from_hms(19, 0, 0, 7, 0, 0).unwrap(),
                                )
                                .toolbox_interval(TimeInterval::from_hms(7, 0, 0, 8, 0, 0).unwrap())
                        })
                        .operational_options(|options| options.number_of_removed_activities(10))
                })
                .operational("TEST_OP-MtnTele-01", |operational_actor| {
                    operational_actor
                        .hours_per_day(6.0)
                        .operational_configuration(|config| {
                            config
                                .add_availability(
                                    Availability::from_rfc3339_strings(
                                        "2025-01-13T07:00:00Z",
                                        "2025-01-27T15:00:00Z",
                                        vec![Asset::Test],
                                    )
                                    .unwrap(),
                                )
                                .add_resource(Resources::MtnTele)
                                .break_interval(TimeInterval::from_hms(11, 0, 0, 12, 0, 0).unwrap())
                                .off_shift_interval(
                                    TimeInterval::from_hms(19, 0, 0, 7, 0, 0).unwrap(),
                                )
                                .toolbox_interval(TimeInterval::from_hms(7, 0, 0, 8, 0, 0).unwrap())
                        })
                        .operational_options(|options| options.number_of_removed_activities(10))
                })
                .operational("TEST_OP-MtnRope-00", |operational_actor| {
                    operational_actor
                        .hours_per_day(6.0)
                        .operational_configuration(|config| {
                            config
                                .add_availability(
                                    Availability::from_rfc3339_strings(
                                        "2025-01-13T07:00:00Z",
                                        "2025-01-27T15:00:00Z",
                                        vec![Asset::Test],
                                    )
                                    .unwrap(),
                                )
                                .add_resource(Resources::MtnRope)
                                .break_interval(TimeInterval::from_hms(11, 0, 0, 12, 0, 0).unwrap())
                                .off_shift_interval(
                                    TimeInterval::from_hms(19, 0, 0, 7, 0, 0).unwrap(),
                                )
                                .toolbox_interval(TimeInterval::from_hms(7, 0, 0, 8, 0, 0).unwrap())
                        })
                        .operational_options(|options| options.number_of_removed_activities(10))
                })
                .operational("TEST_OP-MtnRope-01", |operational_actor| {
                    operational_actor
                        .hours_per_day(6.0)
                        .operational_configuration(|config| {
                            config
                                .add_availability(
                                    Availability::from_rfc3339_strings(
                                        "2025-01-13T07:00:00Z",
                                        "2025-01-27T15:00:00Z",
                                        vec![Asset::Test],
                                    )
                                    .unwrap(),
                                )
                                .add_resource(Resources::MtnRope)
                                .break_interval(TimeInterval::from_hms(11, 0, 0, 12, 0, 0).unwrap())
                                .off_shift_interval(
                                    TimeInterval::from_hms(19, 0, 0, 7, 0, 0).unwrap(),
                                )
                                .toolbox_interval(TimeInterval::from_hms(7, 0, 0, 8, 0, 0).unwrap())
                        })
                        .operational_options(|options| options.number_of_removed_activities(10))
                })
                .operational("TEST_OP-MtnCran-00", |operational_actor| {
                    operational_actor
                        .hours_per_day(6.0)
                        .operational_configuration(|config| {
                            config
                                .add_availability(
                                    Availability::from_rfc3339_strings(
                                        "2025-01-13T07:00:00Z",
                                        "2025-01-27T15:00:00Z",
                                        vec![Asset::Test],
                                    )
                                    .unwrap(),
                                )
                                .add_resource(Resources::MtnCran)
                                .break_interval(TimeInterval::from_hms(11, 0, 0, 12, 0, 0).unwrap())
                                .off_shift_interval(
                                    TimeInterval::from_hms(19, 0, 0, 7, 0, 0).unwrap(),
                                )
                                .toolbox_interval(TimeInterval::from_hms(7, 0, 0, 8, 0, 0).unwrap())
                        })
                        .operational_options(|options| options.number_of_removed_activities(10))
                })
                .operational("TEST_OP-MtnCran-01", |operational_actor| {
                    operational_actor
                        .hours_per_day(6.0)
                        .operational_configuration(|config| {
                            config
                                .add_availability(
                                    Availability::from_rfc3339_strings(
                                        "2025-01-13T07:00:00Z",
                                        "2025-01-27T15:00:00Z",
                                        vec![Asset::Test],
                                    )
                                    .unwrap(),
                                )
                                .add_resource(Resources::MtnCran)
                                .break_interval(TimeInterval::from_hms(11, 0, 0, 12, 0, 0).unwrap())
                                .off_shift_interval(
                                    TimeInterval::from_hms(19, 0, 0, 7, 0, 0).unwrap(),
                                )
                                .toolbox_interval(TimeInterval::from_hms(7, 0, 0, 8, 0, 0).unwrap())
                        })
                        .operational_options(|options| options.number_of_removed_activities(10))
                })
                .operational("TEST_OP-MtnScaf-00", |operational_actor| {
                    operational_actor
                        .hours_per_day(6.0)
                        .operational_configuration(|config| {
                            config
                                .add_availability(
                                    Availability::from_rfc3339_strings(
                                        "2025-01-13T07:00:00Z",
                                        "2025-01-27T15:00:00Z",
                                        vec![Asset::Test],
                                    )
                                    .unwrap(),
                                )
                                .add_resource(Resources::MtnScaf)
                                .break_interval(TimeInterval::from_hms(11, 0, 0, 12, 0, 0).unwrap())
                                .off_shift_interval(
                                    TimeInterval::from_hms(19, 0, 0, 7, 0, 0).unwrap(),
                                )
                                .toolbox_interval(TimeInterval::from_hms(7, 0, 0, 8, 0, 0).unwrap())
                        })
                        .operational_options(|options| options.number_of_removed_activities(10))
                })
                .operational("TEST_OP-MtnScaf-01", |operational_actor| {
                    operational_actor
                        .hours_per_day(6.0)
                        .operational_configuration(|config| {
                            config
                                .add_availability(
                                    Availability::from_rfc3339_strings(
                                        "2025-01-13T07:00:00Z",
                                        "2025-01-27T15:00:00Z",
                                        vec![Asset::Test],
                                    )
                                    .unwrap(),
                                )
                                .add_resource(Resources::MtnScaf)
                                .break_interval(TimeInterval::from_hms(11, 0, 0, 12, 0, 0).unwrap())
                                .off_shift_interval(
                                    TimeInterval::from_hms(19, 0, 0, 7, 0, 0).unwrap(),
                                )
                                .toolbox_interval(TimeInterval::from_hms(7, 0, 0, 8, 0, 0).unwrap())
                        })
                        .operational_options(|options| options.number_of_removed_activities(10))
                })
                .operational("TEST_OP-MtnScaf-02", |operational_actor| {
                    operational_actor
                        .hours_per_day(6.0)
                        .operational_configuration(|config| {
                            config
                                .add_availability(
                                    Availability::from_rfc3339_strings(
                                        "2025-01-13T07:00:00Z",
                                        "2025-01-27T15:00:00Z",
                                        vec![Asset::Test],
                                    )
                                    .unwrap(),
                                )
                                .add_resource(Resources::MtnScaf)
                                .break_interval(TimeInterval::from_hms(11, 0, 0, 12, 0, 0).unwrap())
                                .off_shift_interval(
                                    TimeInterval::from_hms(19, 0, 0, 7, 0, 0).unwrap(),
                                )
                                .toolbox_interval(TimeInterval::from_hms(7, 0, 0, 8, 0, 0).unwrap())
                        })
                        .operational_options(|options| options.number_of_removed_activities(10))
                })
                .operational("TEST_OP-MtnRigg-00", |operational_actor| {
                    operational_actor
                        .hours_per_day(6.0)
                        .operational_configuration(|config| {
                            config
                                .add_availability(
                                    Availability::from_rfc3339_strings(
                                        "2025-01-13T07:00:00Z",
                                        "2025-01-27T15:00:00Z",
                                        vec![Asset::Test],
                                    )
                                    .unwrap(),
                                )
                                .add_resource(Resources::MtnRigg)
                                .break_interval(TimeInterval::from_hms(11, 0, 0, 12, 0, 0).unwrap())
                                .off_shift_interval(
                                    TimeInterval::from_hms(19, 0, 0, 7, 0, 0).unwrap(),
                                )
                                .toolbox_interval(TimeInterval::from_hms(7, 0, 0, 8, 0, 0).unwrap())
                        })
                        .operational_options(|options| options.number_of_removed_activities(10))
                })
                .operational("TEST_OP-MtnRigg-01", |operational_actor| {
                    operational_actor
                        .hours_per_day(6.0)
                        .operational_configuration(|config| {
                            config
                                .add_availability(
                                    Availability::from_rfc3339_strings(
                                        "2025-01-13T07:00:00Z",
                                        "2025-01-27T15:00:00Z",
                                        vec![Asset::Test],
                                    )
                                    .unwrap(),
                                )
                                .add_resource(Resources::MtnRigg)
                                .break_interval(TimeInterval::from_hms(11, 0, 0, 12, 0, 0).unwrap())
                                .off_shift_interval(
                                    TimeInterval::from_hms(19, 0, 0, 7, 0, 0).unwrap(),
                                )
                                .toolbox_interval(TimeInterval::from_hms(7, 0, 0, 8, 0, 0).unwrap())
                        })
                        .operational_options(|options| options.number_of_removed_activities(10))
                })
                .operational("TEST_OP-MtnRigg-02", |operational_actor| {
                    operational_actor
                        .hours_per_day(6.0)
                        .operational_configuration(|config| {
                            config
                                .add_availability(
                                    Availability::from_rfc3339_strings(
                                        "2025-01-13T07:00:00Z",
                                        "2025-01-27T15:00:00Z",
                                        vec![Asset::Test],
                                    )
                                    .unwrap(),
                                )
                                .add_resource(Resources::MtnRigg)
                                .break_interval(TimeInterval::from_hms(11, 0, 0, 12, 0, 0).unwrap())
                                .off_shift_interval(
                                    TimeInterval::from_hms(19, 0, 0, 7, 0, 0).unwrap(),
                                )
                                .toolbox_interval(TimeInterval::from_hms(7, 0, 0, 8, 0, 0).unwrap())
                        })
                        .operational_options(|options| options.number_of_removed_activities(10))
                })
                .operational("TEST_OP-MtnRous-00", |operational_actor| {
                    operational_actor
                        .hours_per_day(6.0)
                        .operational_configuration(|config| {
                            config
                                .add_availability(
                                    Availability::from_rfc3339_strings(
                                        "2025-01-13T07:00:00Z",
                                        "2025-01-27T15:00:00Z",
                                        vec![Asset::Test],
                                    )
                                    .unwrap(),
                                )
                                .add_resource(Resources::MtnRous)
                                .break_interval(TimeInterval::from_hms(11, 0, 0, 12, 0, 0).unwrap())
                                .off_shift_interval(
                                    TimeInterval::from_hms(19, 0, 0, 7, 0, 0).unwrap(),
                                )
                                .toolbox_interval(TimeInterval::from_hms(7, 0, 0, 8, 0, 0).unwrap())
                        })
                        .operational_options(|options| options.number_of_removed_activities(10))
                })
                .operational("TEST_OP-MtnRous-01", |operational_actor| {
                    operational_actor
                        .hours_per_day(6.0)
                        .operational_configuration(|config| {
                            config
                                .add_availability(
                                    Availability::from_rfc3339_strings(
                                        "2025-01-13T07:00:00Z",
                                        "2025-01-27T15:00:00Z",
                                        vec![Asset::Test],
                                    )
                                    .unwrap(),
                                )
                                .add_resource(Resources::MtnRous)
                                .break_interval(TimeInterval::from_hms(11, 0, 0, 12, 0, 0).unwrap())
                                .off_shift_interval(
                                    TimeInterval::from_hms(19, 0, 0, 7, 0, 0).unwrap(),
                                )
                                .toolbox_interval(TimeInterval::from_hms(7, 0, 0, 8, 0, 0).unwrap())
                        })
                        .operational_options(|options| options.number_of_removed_activities(10))
                })
                .operational("TEST_OP-MtnRous-02", |operational_actor| {
                    operational_actor
                        .hours_per_day(6.0)
                        .operational_configuration(|config| {
                            config
                                .add_availability(
                                    Availability::from_rfc3339_strings(
                                        "2025-01-13T07:00:00Z",
                                        "2025-01-27T15:00:00Z",
                                        vec![Asset::Test],
                                    )
                                    .unwrap(),
                                )
                                .add_resource(Resources::MtnRous)
                                .break_interval(TimeInterval::from_hms(11, 0, 0, 12, 0, 0).unwrap())
                                .off_shift_interval(
                                    TimeInterval::from_hms(19, 0, 0, 7, 0, 0).unwrap(),
                                )
                                .toolbox_interval(TimeInterval::from_hms(7, 0, 0, 8, 0, 0).unwrap())
                        })
                        .operational_options(|options| options.number_of_removed_activities(10))
                })
                .operational("TEST_OP-Prodtech-00", |operational_actor| {
                    operational_actor
                        .hours_per_day(6.0)
                        .operational_configuration(|config| {
                            config
                                .add_availability(
                                    Availability::from_rfc3339_strings(
                                        "2025-01-13T07:00:00Z",
                                        "2025-01-27T15:00:00Z",
                                        vec![Asset::Test],
                                    )
                                    .unwrap(),
                                )
                                .add_resource(Resources::Prodtech)
                                .break_interval(TimeInterval::from_hms(11, 0, 0, 12, 0, 0).unwrap())
                                .off_shift_interval(
                                    TimeInterval::from_hms(19, 0, 0, 7, 0, 0).unwrap(),
                                )
                                .toolbox_interval(TimeInterval::from_hms(7, 0, 0, 8, 0, 0).unwrap())
                        })
                        .operational_options(|options| options.number_of_removed_activities(10))
                })
                .operational("TEST_OP-Prodtech-01", |operational_actor| {
                    operational_actor
                        .hours_per_day(6.0)
                        .operational_configuration(|config| {
                            config
                                .add_availability(
                                    Availability::from_rfc3339_strings(
                                        "2025-01-13T07:00:00Z",
                                        "2025-01-27T15:00:00Z",
                                        vec![Asset::Test],
                                    )
                                    .unwrap(),
                                )
                                .add_resource(Resources::Prodtech)
                                .break_interval(TimeInterval::from_hms(11, 0, 0, 12, 0, 0).unwrap())
                                .off_shift_interval(
                                    TimeInterval::from_hms(19, 0, 0, 7, 0, 0).unwrap(),
                                )
                                .toolbox_interval(TimeInterval::from_hms(7, 0, 0, 8, 0, 0).unwrap())
                        })
                        .operational_options(|options| options.number_of_removed_activities(10))
                })
                .operational("TEST_OP-Prodtech-02", |operational_actor| {
                    operational_actor
                        .hours_per_day(6.0)
                        .operational_configuration(|config| {
                            config
                                .add_availability(
                                    Availability::from_rfc3339_strings(
                                        "2025-01-13T07:00:00Z",
                                        "2025-01-27T15:00:00Z",
                                        vec![Asset::Test],
                                    )
                                    .unwrap(),
                                )
                                .add_resource(Resources::Prodtech)
                                .break_interval(TimeInterval::from_hms(11, 0, 0, 12, 0, 0).unwrap())
                                .off_shift_interval(
                                    TimeInterval::from_hms(19, 0, 0, 7, 0, 0).unwrap(),
                                )
                                .toolbox_interval(TimeInterval::from_hms(7, 0, 0, 8, 0, 0).unwrap())
                        })
                        .operational_options(|options| options.number_of_removed_activities(10))
                })
        })
}
