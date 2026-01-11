use chrono::Utc;

pub mod db;
pub mod extractors;
pub mod models;
pub mod provider;
pub mod routes;

pub fn current_timestamp() -> i64
{
    Utc::now().timestamp()
}
