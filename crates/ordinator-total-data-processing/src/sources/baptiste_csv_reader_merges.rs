use std::collections::BTreeMap;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::Mutex;

use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use chrono::NaiveDate;
use chrono::NaiveTime;
use ordinator_configuration::toml_baptiste::BaptisteToml;
use ordinator_scheduling_environment::work_order;
use ordinator_scheduling_environment::work_order::operation::Operation;
use ordinator_scheduling_environment::work_order::operation::Operations;
use ordinator_scheduling_environment::work_order::work_order_dates::unloading_point::UnloadingPoint;
use ordinator_scheduling_environment::work_order::work_order_info::priority::Priority;
use ordinator_scheduling_environment::work_order::work_order_info::work_order_text::WorkOrderText;
use ordinator_scheduling_environment::work_order::work_order_info::work_order_type::WorkOrderType;
use ordinator_scheduling_environment::work_order::work_order_info::WorkOrderInfoBuilder;
use ordinator_scheduling_environment::work_order::WorkOrder;
use ordinator_scheduling_environment::work_order::WorkOrderNumber;
use ordinator_scheduling_environment::work_order::WorkOrders;
use ordinator_scheduling_environment::worker_environment::resources::Skill;
use rayon::prelude::*;

use super::baptiste_csv_reader::populate_csv_structures;
use super::baptiste_csv_reader::FLOCTechnicaID;
use super::baptiste_csv_reader::FunctionalLocationsCsv;
use super::baptiste_csv_reader::OperationsStatusCsv;
use super::baptiste_csv_reader::OperationsStatusCsvAggregated;
use super::baptiste_csv_reader::WorkCenterCsv;
use super::baptiste_csv_reader::WorkOperations;
use super::baptiste_csv_reader::WorkOperationsCsv;
use super::baptiste_csv_reader::WorkOrdersCsv;
use super::baptiste_csv_reader::WorkOrdersStatusCsv;
use super::baptiste_csv_reader::WorkOrdersStatusCsvAggregated;
use super::baptiste_csv_reader::WBSID;
use crate::sap_mapper_and_types::DATS;
use crate::sap_mapper_and_types::TIMS;

// TODO: Insert main configuration here; `operating time` is crucial.
pub fn load_csv_data(file_path: &BaptisteToml) -> Result<WorkOrders>
{
    let functional_locations_csv =
        populate_csv_structures::<FunctionalLocationsCsv>(&file_path.mid_functional_locations)
            .with_context(||format!("CSV file from SAP extract is not available at: {}", &file_path.mid_functional_locations.display()))?;

    let operations_status_csv =
        populate_csv_structures::<OperationsStatusCsv>(&file_path.mid_operations_status)
            .with_context(||format!("CSV file from SAP extract is not available at: {}", &file_path.mid_operations_status.display()))?;

    let work_center_csv = populate_csv_structures::<WorkCenterCsv>(&file_path.mid_work_center)
            .with_context(||format!("CSV file from SAP extract is not available at: {}", &file_path.mid_work_center.display()))?;

    let work_operations_csv =
        populate_csv_structures::<WorkOperationsCsv>(&file_path.mid_work_operations)
            .with_context(||format!("CSV file from SAP extract is not available at: {}", &file_path.mid_work_operations.display()))?;

    let work_orders_csv = populate_csv_structures::<WorkOrdersCsv>(&file_path.mid_work_orders)
            .with_context(||format!("CSV file from SAP extract is not available at: {}", &file_path.mid_work_orders.display()))?;

    let work_orders_status_csv =
        populate_csv_structures::<WorkOrdersStatusCsv>(&file_path.mid_work_orders_status)
            .with_context(||format!("CSV file from SAP extract is not available at: {}", &file_path.mid_work_orders_status.display()))?;

    let work_orders_status_agg = WorkOrdersStatusCsvAggregated::new(work_orders_status_csv.clone());

    let operations_status_agg = OperationsStatusCsvAggregated::new(operations_status_csv.clone());

    let work_operations = WorkOperations::new(&work_orders_csv, &work_operations_csv);

    let work_orders_inner = create_work_orders(
        functional_locations_csv.clone(),
        operations_status_agg,
        work_center_csv.clone(),
        work_operations,
        work_orders_csv.clone(),
        work_orders_status_agg,
    )
    .with_context(|| format!("File {file_path:#?} could not be found while loading data",))?;

    let work_orders = WorkOrders::builder()
        .work_orders_manual(work_orders_inner)
        .subnetwork_manual(HashMap::default())
        .build();
    Ok(work_orders)
}

#[allow(dead_code)]
#[allow(non_snake_case)]
fn create_work_orders(
    functional_locations: HashMap<FLOCTechnicaID, FunctionalLocationsCsv>,
    _operations_status: OperationsStatusCsvAggregated,
    work_center: HashMap<WBSID, WorkCenterCsv>,
    work_operations_csv: WorkOperations,
    work_orders: HashMap<WorkOrderNumber, WorkOrdersCsv>,
    work_orders_status: WorkOrdersStatusCsvAggregated,
) -> Result<HashMap<WorkOrderNumber, WorkOrder>>
{
    assert!(!work_operations_csv.inner.is_empty());

    let arc_mutex_inner_work_orders = Arc::new(Mutex::new(HashMap::new()));

    work_orders.into_iter().filter(|e| work_operations_csv.inner.contains_key(&e.0)).collect::<HashMap<_,_>>().par_iter().for_each(|(work_order_number, work_order_csv): (&WorkOrderNumber, &WorkOrdersCsv)|   {
        let main_work_center: Skill = Skill::from_str(
            work_center
                .get(&work_order_csv.WO_WBS_ID)
                .unwrap()
                .WBS_Name.as_str()
        ).unwrap();

        let functional_location =
            &functional_locations.get(&work_order_csv.WO_Functional_Location_Number);

        let functional_location = match functional_location {
            Some(functional_location_csv) => &functional_location_csv.FLOC_Name,
            None => "WARN: FUNCTIONAL_LOCATION MISSING IS THIS CORRECT?",
        };

        // TODO: Refactor to use a builder pattern for WorkOrderText
        let work_order_text = WorkOrderText::new(
            None,
            None,
            work_order_csv.WO_Header_Description.clone(),
            None,
            None,
            None,
        );

        let work_order_info_detail = work_order::work_order_info::WorkOrderInfoDetail::new(
            work_order_csv.WO_SubNetwork_ID.clone(),
            work_order_csv.WO_Plan_Maintenance_Number.clone(),
            work_order_csv.WO_Planner_Group.clone(),
            work_order_csv.WO_Maintenance_Plan_Name.clone(),
            "PM_COLLECTIVE_MISSING_TODO".to_string(),
            "ROOM_MISSING_TODO".to_string(),
        );

        let priority = Priority::dyn_new(Box::new(work_order_csv.WO_Priority.clone()));

        let work_order_type = WorkOrderType::new(&work_order_csv.WO_Order_Type, priority.clone())
            .expect("Invalid WorkOrderType's should have been filtered out");

        
        let operations: Operations = work_operations_csv
            .inner
            .get(work_order_number)
            .with_context(|| format!("What should be done to fix this? Does it make sense if there are no available `Operations`?\n{work_order_number:#?}")).unwrap()
            .iter()
            .map(|(operations_number, operation_csv)| -> Result<(u64, Operation)> {
                let resource =
                    Skill::from_str(&work_center.get(&operation_csv.OPR_WBS_ID).unwrap().WBS_Name).map_err(|e| anyhow!(e))?;

                // TODO: Extract unloading point initialization to a separate function to avoid state duplication
                let unloading_point: UnloadingPoint =
                    UnloadingPoint::new(operation_csv.OPR_Scheduled_Work.clone(), None);

                let planned_work
                    = operation_csv.OPR_Planned_Work.clone().parse::<f64>().expect("Planned work should be present. There is not implemented correct error handling here due to `rayon::par_iter`");

                let actual_work = operation_csv.OPR_Actual_Work.clone().parse::<f64>().unwrap_or_default();

                // Calculate remaining work, handling negative values from data inconsistencies
                let remaining_work = {
                    let work_remaining = operation_csv.OPR_Planned_Work.clone().parse::<f64>().unwrap_or_default() - operation_csv.OPR_Actual_Work.clone().parse::<f64>().unwrap_or_default();
                    if work_remaining < 0.0 {
                       0.0 
                    } else {
                        work_remaining 
                    }
                };

                // Convert SAP DATS format to NaiveDate for start and end datetimes
                let naive_start_DATS: NaiveDate = DATS(operation_csv.OPR_Start_Date.clone()).try_into().expect("The OPR_Start_Date should have been filtered out, we should not experience this error.");
                let naive_start_TIMS: NaiveTime = TIMS(operation_csv.OPR_Start_Time.clone()).into();

                let naive_end_DATS: NaiveDate = DATS(operation_csv.OPR_End_Date.clone()).try_into().expect("The OPR_End_Date should have been filtered out, we should not experience this error.");
                let naive_end_TIMS: NaiveTime = TIMS(operation_csv.OPR_End_Time.clone()).into();

                let naive_start_datetime = naive_start_DATS.and_time(naive_start_TIMS);
                let naive_end_datetime = naive_end_DATS.and_time(naive_end_TIMS);

                let utc_start_datetime = naive_start_datetime.and_utc();
                let utc_end_datetime = naive_end_datetime.and_utc();

                // Build operation with resource and timing information
                let operation = Operation::builder(*operations_number, resource)
                    .unloading_point(unloading_point)
                    .operation_description(operation_csv.OPR_Description.clone())
                    .operation_info(|oib| {
                        oib
                            .number(operation_csv.OPR_Workers_Numbers)
                            .work_remaining(remaining_work)
                            .work_actual(actual_work)
                            .work(planned_work)
                    })

                    .operation_analytic(|oab| {
                        oab.preparation_time(1.0)
                    })
                    .operation_dates(|odb| {
                        odb.earliest_start_datetime(utc_start_datetime)
                            .earliest_finish_datetime(utc_end_datetime)
                    }).build();

                Ok((*operations_number, operation))
                
            }).collect::<Result<BTreeMap<u64, Operation>>>().map_err(|e| anyhow!(e)).expect("This is not the best way of making error handling")
            .into();

        let status_codes_string = work_orders_status.inner
            .get(&work_order_csv.WO_Status_ID)
            .expect("Should always be present");

        if !status_codes_string.contains("REL") {
            return;
        }; 

        let earliest_allowed_start_date: NaiveDate =
            DATS(work_order_csv.WO_Earliest_Allowed_Start_Date.clone())
                .try_into()
                .expect("The WorkOrders that have invalid EASD are filtered out");

        let latest_allowed_finish_date: NaiveDate =
            DATS(work_order_csv.WO_Latest_Allowed_Finish_Date.clone())
                .try_into()
                .expect("The WorkOrders that have invalid EASD are filtered out");

        let basic_start_date: NaiveDate = DATS(work_order_csv.WO_Basic_Start_Date.clone())
            .try_into()
            .expect("The WorkOrders that have invalid EASD are filtered out");

        let basic_finish_date: NaiveDate = DATS(work_order_csv.WO_Basic_End_Date.clone())
            .try_into()
            .expect("The WorkOrders that have invalid EASD are filtered out");

        let duration = basic_finish_date - basic_start_date;

        let work_order = WorkOrder::builder(*work_order_number)
            .main_work_center(main_work_center)
            .operations(operations)
            .work_order_info_builder(|woi: WorkOrderInfoBuilder| -> WorkOrderInfoBuilder {
                woi.priority(priority)
                    .work_order_type(work_order_type)
                    .functional_location_from_str(functional_location)
                    .work_order_text(work_order_text)
                    .revision_from_str(&work_order_csv.WO_Revision)
                    .system_condition_from_str(&work_order_csv.WO_System_Condition).expect("If this fails consider making stronger error handling")
                    .work_order_info_detail(work_order_info_detail)
            })
            .work_order_analytic_builder(|woab| {
                woab.system_status_codes(|con| con.from_str(status_codes_string))
                    .user_status_codes(|con| con.from_str(status_codes_string))
            })
            .work_order_dates_builder(|wodb| {
                wodb.earliest_allowed_start_date(earliest_allowed_start_date)
                    .latest_allowed_finish_date(latest_allowed_finish_date)
                    .basic_start_date(basic_start_date)
                    .basic_finish_date(basic_finish_date)
                    .duration(duration)
            })
            .build();

        // TODO: Validate work order dates against MaterialToPeriod
        arc_mutex_inner_work_orders.lock().unwrap().insert(*work_order_number, work_order);
    });
    let work_orders = arc_mutex_inner_work_orders.lock().unwrap().clone();
    Ok(work_orders)
}
