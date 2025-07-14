use anyhow::Context;

use crate::time_environment::period::Period;
use crate::work_order::WorkOrderNumber;
use crate::work_order::WorkOrders;

/// TODO WARN [ ] 2025-07-14 `UnloadingPoint` is Total Specific
/// This is an endpoint for setting the `UnloadingPoint` for the
/// entire `WorkOrder` in every `Operation`
impl WorkOrders
{
    pub fn update_period_unloading_point(
        &mut self,
        work_order_number: &WorkOrderNumber,
        period: &Period,
    ) -> anyhow::Result<()>
    {
        self.inner
            .get_mut(work_order_number)
            .with_context(|| {
                format!(
                    "work order number:\n{work_order_number}\nwas not found in the SchedulingEnvironment"
                )
            })?
            .operations
            .0
            .iter_mut()
            .for_each(|e| e.1.unloading_point.period_string = Some(period.period_string()));

        Ok(())
    }
}
