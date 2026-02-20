use ordinator_orchestrator::Asset;
use ordinator_orchestrator::Availability;
use ordinator_orchestrator::Skill;
use ordinator_scheduling_environment::time_environment::TimeInterval;
use ordinator_scheduling_environment::worker_environment::ActorSpecificationBuilder;

pub fn three_workers_builder(actor_builder: ActorSpecificationBuilder)
-> ActorSpecificationBuilder
{
    actor_builder
        .weekly(|weekly| {
            weekly
                .id("TEST_STRATEGIC")
                .number_of_weekly_periods(52)
                .weekly_options(|f| {
                    f.number_of_removed_work_orders(5)
                        .urgency_weight(1000)
                        .resource_penalty_weight(1_000_000)
                        .clustering_weight(1000)
                })
        })
        .project(|project| {
            project
                .id("TEST_TACTICAL")
                .number_of_project_days(120)
                .project_options(|f| {
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
                                    Availability::from_rfc3339_strings(
                                        "2025-01-13T07:00:00Z",
                                        "2025-01-27T15:00:00Z",
                                        vec![Asset::Test],
                                    )
                                    .unwrap(),
                                )
                                .add_resource(Skill::MtnMech)
                                .break_interval(TimeInterval::from_hms(11, 0, 0, 12, 0, 0).unwrap())
                                .off_shift_interval(
                                    TimeInterval::from_hms(19, 0, 0, 7, 0, 0).unwrap(),
                                )
                                .toolbox_interval(TimeInterval::from_hms(7, 0, 0, 8, 0, 0).unwrap())
                        })
                        .operational_options(|options| options.number_of_removed_activities(10))
                })
                .operational("TEST_OP-001-02", |operational_actor_2| {
                    operational_actor_2
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
                                .add_resource(Skill::MtnElec)
                                // Do not expose the three internal types; they should be encapsulated
                                .break_interval(TimeInterval::from_hms(11, 0, 0, 12, 0, 0).unwrap())
                                .off_shift_interval(
                                    TimeInterval::from_hms(19, 0, 0, 7, 0, 0).unwrap(),
                                )
                                .toolbox_interval(TimeInterval::from_hms(7, 0, 0, 8, 0, 0).unwrap())
                        })
                        .operational_options(|options| options.number_of_removed_activities(10))
                })
                .operational("TEST_OP-002-01", |operational_actor_2| {
                    operational_actor_2
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
                                // .add_resource(Resources::MtnInst)
                                .add_resource(Skill::MtnInst)
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
