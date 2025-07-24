use anyhow::Context;
use chrono::NaiveDate;
use ordinator_operational_actor::algorithm::operational_solution::OperationalSolution;
use ordinator_orchestrator_actor_traits::SystemSolution;
use ordinator_scheduling_environment::Asset;
use ordinator_scheduling_environment::work_order::WorkOrderNumber;
use ordinator_scheduling_environment::work_order::operation::ActivityNumber;
use ordinator_scheduling_environment::worker_environment::resources::Id;
use ordinator_strategic_actor::algorithm::strategic_solution::StrategicSolution;
use ordinator_supervisor_actor::algorithm::supervisor_solution::SupervisorSolution;
use ordinator_tactical_actor::algorithm::tactical_solution::TacticalSolution;
use serde::Deserialize;
use serde::Serialize;
use strum::IntoEnumIterator;
use ts_rs::TS;
use utoipa::ToSchema;

pub mod orchestrator;
pub mod scheduler;
pub mod supervisor;
pub mod technician;
// This is a DTO object, it should be moved out of the
// `scheduling-environment`
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, ToSchema, TS)]
pub struct AssetNames(String);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, ToSchema, TS)]
#[ts(export)]
pub struct PeriodDto(pub String);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, ToSchema, TS)]
#[ts(export)]
pub struct NaiveDateDto(pub String);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, ToSchema, TS)]
pub struct WorkOrderNumberDto(pub u64);

impl From<WorkOrderNumberDto> for WorkOrderNumber
{
    fn from(value: WorkOrderNumberDto) -> Self
    {
        WorkOrderNumber(value.0)
    }
}
impl AssetNames
{
    pub fn convert_to_asset_names() -> Vec<AssetNames>
    {
        let mut vec = Vec::new();
        for asset in Asset::iter() {
            let asset_name = AssetNames(asset.to_string());
            vec.push(asset_name);
        }
        vec
    }
}

impl From<Asset> for AssetNames
{
    fn from(value: Asset) -> Self
    {
        let value = value.to_string();

        Self(value)
    }
}
impl TryFrom<AssetNames> for Asset
{
    type Error = anyhow::Error;

    fn try_from(value: AssetNames) -> anyhow::Result<Self>
    {
        Asset::new_from_string(&value.0)
            .with_context(|| format!("This operation should never fail\nAssetNames: {value:#?}"))
    }
}

impl From<NaiveDate> for NaiveDateDto
{
    fn from(value: NaiveDate) -> Self
    {
        Self(value.to_string())
    }
}

impl TryFrom<NaiveDateDto> for NaiveDate
{
    type Error = chrono::ParseError;

    fn try_from(value: NaiveDateDto) -> Result<Self, Self::Error>
    {
        let naive_date = NaiveDate::parse_from_str(&value.0, "%y-%m-%s")?;
        Ok(naive_date)
    }
}

// TODO [ ]
// Add dependencies for each of these
pub type TotalSystemSolution =
    SystemSolution<StrategicSolution, TacticalSolution, SupervisorSolution, OperationalSolution>;

#[derive(PartialEq, Eq, PartialOrd, Ord, ToSchema, Serialize)]
pub struct IdDto
{
    id: String,
    resources: Vec<String>,
    asset: Vec<AssetNames>,
}

impl From<Id> for IdDto
{
    fn from(value: Id) -> Self
    {
        Self {
            id: value.0,
            resources: value.1.iter().map(|e| e.to_string()).collect(),
            asset: value.2.iter().map(|e| AssetNames(e.to_string())).collect(),
        }
    }
}
#[derive(ToSchema, Serialize)]
struct WorkOrderActivityDto
{
    work_order: u64,
    activity: u64,
}

impl From<(WorkOrderNumber, ActivityNumber)> for WorkOrderActivityDto
{
    fn from(value: (WorkOrderNumber, ActivityNumber)) -> Self
    {
        Self {
            work_order: value.0.0,
            activity: value.1,
        }
    }
}
