use chrono::Utc;

pub mod db;
pub mod jwt;
pub mod models;
pub mod provider;
pub mod routes;
pub mod extractors;

pub fn current_timestamp() -> i64
{
    Utc::now().timestamp()
}
