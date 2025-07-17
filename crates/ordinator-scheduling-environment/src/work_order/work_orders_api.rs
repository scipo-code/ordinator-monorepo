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
            // This is a little bit bummer. I think that the best thing to do here is to make the
            // code work correctly with the supplied 
            // This is not okay. I think that there are many different paths to take here. 
            .for_each(|e| {
                e.1.unloading_point = UnloadingPoint::new(e.1.unloading_point.string.clone(), Some(period.period_string()))
            });

        Ok(())
    }
}
