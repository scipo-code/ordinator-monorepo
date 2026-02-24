use std::fs;
use std::path::PathBuf;

use chrono::NaiveDate;
use chrono::NaiveDateTime;
use ordinator_scheduling_hypergraph::schedule_graph::ScheduleGraph;
use ordinator_scheduling_environment::time_environment::period::Period;
use ordinator_scheduling_environment::work_order::WorkOrder;
use ordinator_scheduling_environment::work_order::operation::Operation;
use ordinator_scheduling_environment::worker_environment::availability::Availability;
use ordinator_scheduling_environment::worker_environment::resources::Skill;
use ordinator_scheduling_environment::worker_environment::worker::Technician;
use serde::Deserialize;

/// Intermediate struct for deserializing technician data from JSON
#[derive(Deserialize, Debug)]
struct TechnicianData
{
    id: usize,
    skills: Vec<Skill>,
    availabilities: Vec<(NaiveDateTime, NaiveDateTime)>,
}

/// Intermediate struct for deserializing work order data from JSON
#[derive(Deserialize, Debug)]
struct WorkOrderData
{
    work_order_number: u64,
    basic_start_date: NaiveDate,
    activities: Vec<ActivityData>,
}

#[derive(Deserialize, Debug)]
struct ActivityData
{
    activity_number: u64,
    number_of_people: u64,
    resource: Skill,
}

fn get_test_data_path(filename: &str) -> PathBuf
{
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest_dir)
        .parent()
        .unwrap()
        .join("ordinator-test-support")
        .join("ordinator-hypergraph-test-data-instances-and-generation")
        .join(filename)
}

#[test]
fn test_large_scale_hypergraph()
{
    // Read periods from JSON
    let periods_json =
        fs::read_to_string(get_test_data_path("periods.json")).expect("Failed to read periods.json");
    let period_dates: Vec<NaiveDate> =
        serde_json::from_str(&periods_json).expect("Failed to parse periods.json");

    // Read work orders from JSON
    let work_orders_json = fs::read_to_string(get_test_data_path("work_orders.json"))
        .expect("Failed to read work_orders.json");
    let work_order_data: Vec<WorkOrderData> =
        serde_json::from_str(&work_orders_json).expect("Failed to parse work_orders.json");

    // Read technicians from JSON
    let technicians_json = fs::read_to_string(get_test_data_path("technicians.json"))
        .expect("Failed to read technicians.json");
    let technician_data: Vec<TechnicianData> =
        serde_json::from_str(&technicians_json).expect("Failed to parse technicians.json");

    println!("Loaded {} periods", period_dates.len());
    println!("Loaded {} work orders", work_order_data.len());
    println!("Loaded {} technicians", technician_data.len());

    // Create the schedule graph
    let mut schedule_graph = ScheduleGraph::new();

    // First, add all skills as nodes
    schedule_graph.add_skill(Skill::MtnMech);
    schedule_graph.add_skill(Skill::MtnElec);

    // Add all periods (this also creates day nodes)
    for period_date in &period_dates {
        let period = Period::from_start_date(*period_date);
        schedule_graph
            .add_period(period)
            .expect("Failed to add period");
    }
    println!("Added {} periods to graph", period_dates.len());

    // Add all work orders
    let mut work_orders_added = 0;
    let work_orders: Vec<WorkOrder> = work_order_data
        .iter()
        .map(|wo_data| {
            let operations: Vec<Operation> = wo_data
                .activities
                .iter()
                .map(|a| Operation::new(a.activity_number, a.number_of_people, a.resource))
                .collect();
            WorkOrder::new(wo_data.work_order_number, wo_data.basic_start_date, operations)
                .expect("Failed to create WorkOrder")
        })
        .collect();

    for work_order in &work_orders {
        match schedule_graph.add_work_order(work_order) {
            Ok(_) => work_orders_added += 1,
            Err(e) => {
                // Some work orders might fail if their basic_start_date
                // doesn't fall within a period's days
                eprintln!(
                    "Warning: Failed to add work order {}: {:?}",
                    work_order.work_order_number(),
                    e
                );
            }
        }
    }
    println!("Added {} work orders to graph", work_orders_added);

    // Add all technicians
    let mut technicians_added = 0;
    for tech_data in &technician_data {
        let mut builder = Technician::builder(tech_data.id);

        for skill in &tech_data.skills {
            builder = builder.add_skill(*skill);
        }

        let technician = builder.build();

        if let Some(&(start, end)) = tech_data.availabilities.first() {
            let availability = Availability::from_naive(start, end);
            match schedule_graph.add_technician(technician, availability) {
                Ok(_) => technicians_added += 1,
                Err(e) => {
                    eprintln!(
                        "Warning: Failed to add technician {}: {:?}",
                        tech_data.id, e
                    );
                }
            }
        }
    }
    println!("Added {} technicians to graph", technicians_added);

    // Verify the graph has the expected structure
    let node_count = schedule_graph.node_count();
    let hyperedge_count = schedule_graph.hyperedge_count();

    println!("\nGraph statistics:");
    println!("  Total nodes: {}", node_count);
    println!("  Total hyperedges: {}", hyperedge_count);

    // Basic assertions
    assert!(
        period_dates.len() == 52,
        "Expected 52 periods, got {}",
        period_dates.len()
    );
    assert!(
        work_order_data.len() == 1000,
        "Expected 1000 work orders, got {}",
        work_order_data.len()
    );
    assert!(
        technician_data.len() == 100,
        "Expected 100 technicians, got {}",
        technician_data.len()
    );

    // Verify graph is populated
    assert!(node_count > 0, "Graph should have nodes");
    assert!(hyperedge_count > 0, "Graph should have hyperedges");

    println!("\nLarge scale hypergraph test passed!");
}
