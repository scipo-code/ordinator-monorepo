use ordinator_scheduling_environment::work_order::WorkOrderNumber;
use ordinator_scheduling_environment::work_order::work_order_info::work_order_type::WorkOrderType;
use ordinator_scheduling_environment::worker_environment::resources::Skill;
use serde::Deserialize;

pub mod work_orders_0100_work_orders_04_resource_types;
pub mod work_orders_0400_work_orders_04_resource_types;
pub mod work_orders_0400_work_orders_11_resource_types;
pub mod work_orders_1600_work_orders_11_resource_types;
pub mod work_orders_6400_work_orders_11_resource_types;

#[derive(Clone, Deserialize)]
pub struct WorkOrderData
{
    work_order_number: WorkOrderNumber,
    #[allow(dead_code)]
    priority: WorkOrderType,
    operations: Vec<OperationInput>,
    basic_start: (i32, u32, u32),
    basic_finish: (i32, u32, u32),
    easd: (i32, u32, u32),
    lafd: (i32, u32, u32),
    codes: (bool, bool),
}

#[derive(Clone, Deserialize)]
pub struct OperationInput
{
    activity: u64,
    work_remaining: f64,
    early_start: (i32, u32, u32, u32, u32, u32),
    early_finish: (i32, u32, u32, u32, u32, u32),
    preparation: f64,
    resource: Skill,
}
