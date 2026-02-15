use anyhow::Context;

use super::work_order_dates::unloading_point::UnloadingPoint;
use crate::time_environment::period::Period;
use crate::work_order::WorkOrderNumber;
use crate::work_order::WorkOrders;

/// Updates the `UnloadingPoint` for the entire `WorkOrder` across all operations.
///
/// TODO WARN [ ] 2025-07-14 `UnloadingPoint` is site-specific
impl WorkOrders
{
    pub fn update_scheduled_period(
        &mut self,
        work_order_number: &WorkOrderNumber,
        period: &Period,
        periods: &[Period],
    ) -> anyhow::Result<()>
    {
        let work_order = self
            .inner
            .get_mut(work_order_number)
            .with_context(|| "{:#?} not found in SchedulingEnvironment")?;

        if period == periods.first().context("There is no first period")? {
            work_order.work_order_analytic.user_status_codes.schedule();
        } else if period == periods.get(1).context("There is no second period")? {
            work_order.work_order_analytic.user_status_codes.draft();
        } else {
            work_order
                .work_order_analytic
                .user_status_codes
                .out_of_scheduled_or_draft();
        }

        work_order.work_order_dates.basic_start_date = period.start_datetime().date_naive();
        work_order.work_order_dates.basic_finish_date = period.finish_datetime().date_naive();

        work_order
            .operations
            .0
            .iter_mut()
            // Update the unloading point with the new period for each operation
            .for_each(|e| {
                e.1.unloading_point = UnloadingPoint::new(
                    e.1.unloading_point.string.clone(),
                    Some(period.period_string()),
                )
            });

        Ok(())
    }
}
