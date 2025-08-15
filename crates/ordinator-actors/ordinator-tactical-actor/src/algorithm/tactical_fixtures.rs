use chrono::NaiveDate;
use chrono::TimeZone;
use chrono::Utc;
use ordinator_orchestrator_actor_traits::Parameters;
use ordinator_scheduling_environment::time_environment::day::Day;
use ordinator_scheduling_environment::work_order::WorkOrderNumber;
use ordinator_scheduling_environment::work_order::operation::Work;
use ordinator_scheduling_environment::worker_environment::resources::Resources;

use super::tactical_parameters::TacticalParameters;

pub fn tactical_parameters_1() -> TacticalParameters
{
    let start_date = Day(NaiveDate::from_ymd_opt(2025, 1, 1).unwrap());
    TacticalParameters::from_builder()
        .tactical_work_orders_builder(WorkOrderNumber(20002), |tac_wo_bui| {
            tac_wo_bui
                .weight(1000)
                .earliest_allowed_start_date(NaiveDate::from_ymd_opt(2025, 2, 1).unwrap())
                .tactical_operation_parameters(|tac_ope_par_bui| {
                    tac_ope_par_bui.add_operation_parameter_with_builder(10, |ope_par_bui| {
                        ope_par_bui
                            .number(1)
                            .duration(Work::from(2.0))
                            .operating_time(Work::from(6.0))
                            .resource(Resources::MtnMech)
                            .forced_start_date(None)
                            .earliest_start_datetime(
                                Utc.with_ymd_and_hms(2025, 1, 12, 7, 0, 0).unwrap().to_utc(),
                            )
                            .earliest_finish_datetime(
                                Utc.with_ymd_and_hms(2025, 1, 12, 9, 0, 0).unwrap().to_utc(),
                            )
                    })
                })
        })
        .tactical_days(Day::range(start_date, 20))
        .tactical_capacity_builder(|resource_builder| {
            resource_builder
                .add_resource_uniform(Resources::MtnMech, Work::from(6.0), 20)
                .add_resource_uniform(Resources::MtnElec, Work::from(6.0), 20)
                .add_resource_uniform(Resources::MtnInst, Work::from(6.0), 20)
        })
        .tactical_options_builder(|tac_opt_bui| {
            tac_opt_bui
                .number_of_removed_work_orders(15)
                .urgency(1000)
                .resource_penalty(1_000_000)
        })
        .build()
}
