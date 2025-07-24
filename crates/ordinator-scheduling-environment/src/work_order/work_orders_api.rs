use anyhow::Context;

use super::work_order_dates::unloading_point::UnloadingPoint;
use crate::time_environment::period::Period;
use crate::work_order::WorkOrderNumber;
use crate::work_order::WorkOrders;

/// TODO WARN [ ] 2025-07-14 `UnloadingPoint` is Total Specific
/// This is an endpoint for setting the `UnloadingPoint` for the
/// entire `WorkOrder` in every `Operation`
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

        work_order.work_order_dates.basic_start_date = period.start_date().date_naive();
        work_order.work_order_dates.basic_finish_date = period.finish_date().date_naive();

        work_order
            .operations
            .0
            .iter_mut()
            // This is a little bit bummer. I think that the best thing to do here is to make the
            // code work correctly with the supplied
            // This is not okay. I think that there are many different paths to take here.
            .for_each(|e| {
                e.1.unloading_point = UnloadingPoint::new(
                    e.1.unloading_point.string.clone(),
                    Some(period.period_string()),
                )
            });

        Ok(())
    }
}
