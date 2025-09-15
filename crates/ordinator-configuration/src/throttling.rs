use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Deserialize, Debug)]
pub struct Throttling
{
    pub strategic_throttling: u64,
    pub tactical_throttling: u64,
    pub supervisor_throttling: u64,
    pub operational_throttling: u64,
}
