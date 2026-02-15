use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;

use anyhow::Context;
use anyhow::Result;
use arc_swap::ArcSwap;
use chrono::DateTime;
use chrono::Utc;
use ordinator_configuration::SystemConfigurations;
use ordinator_scheduling_environment::Asset;
use ordinator_scheduling_environment::SchedulingEnvironment;
use ordinator_total_data_processing::sources::baptiste_csv_reader_merges::load_csv_data;

pub struct DataBaseConnection;

// TODO: Refactor to maintain continuous MongoDB connection instead of loading all data at once
impl DataBaseConnection
{
    pub fn scheduling_environment(
        current_time: DateTime<Utc>,
        asset: Asset,
        system_configurations: Arc<ArcSwap<SystemConfigurations>>,
    ) -> Result<Arc<Mutex<SchedulingEnvironment>>>
    {
        let path_to_time_environment = PathBuf::from(
            "./temp_scheduling_environment_database/time_environment/time_input.toml",
        );

        let path_to_work_orders = system_configurations.load().temp_database_path.clone();
        let csv_data_locations = &system_configurations.load().data_locations;

        let asset_string = asset.to_string().to_lowercase();
        let path = format!(
            "temp_scheduling_environment_database/actor_specifications/actor_specification_{asset_string}.toml",
        );
        #[cfg(test)]
        let path_to_workers = Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
        #[cfg(not(test))]
        let path_to_workers = Path::new("./").join(path);

        let database_path = &system_configurations.load().temp_database_path;

        let path_to_work_order_policies = PathBuf::from(format!(
            "temp_scheduling_environment_database/work_order_policies/work_order_policies_{asset_string}.toml",
        ));
        let path_to_material_to_period = PathBuf::from(
            "temp_scheduling_environment_database/material_repo/material_to_period.toml",
        );

        if database_path.exists() {
            SchedulingEnvironment::builder()
                .work_orders_from_json(path_to_work_orders)?
                .time_environment_from_toml(path_to_time_environment, current_time)?
                .worker_environment_from_toml(path_to_workers, asset)?
                .work_order_policies_from_toml(path_to_work_order_policies)?
                .material_repo_from_toml(path_to_material_to_period)?
                .build()
        } else {
            let work_orders = load_csv_data(csv_data_locations).with_context(|| {
                format!("SchedulingEnvironment could not be built from {csv_data_locations:#?}",)
            })?;

            let work_orders_json = serde_json::to_string(&work_orders)
                .context("Could not make a string value out of WorkOrders")?;

            let mut file = File::create(path_to_work_orders)
                .context("Could not create work_orders.json file")?;

            file.write_all(work_orders_json.as_bytes())
                .context("Could not write bytes to work_orders.json file")?;

            SchedulingEnvironment::builder()
                .worker_environment_from_toml(path_to_workers, asset)?
                .time_environment_from_toml(path_to_time_environment, current_time)?
                .work_orders(work_orders)
                .work_order_policies_from_toml(path_to_work_order_policies)?
                .material_repo_from_toml(path_to_material_to_period)?
                .build()
        }
    }
}
