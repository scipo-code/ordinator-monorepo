use anyhow::Context;
use chrono::DateTime;
use chrono::NaiveDate;
use chrono::Utc;
use ordinator_operational_actor::algorithm::operational_solution::OperationalSolution;
use ordinator_orchestrator_actor_traits::SystemSolution;
use ordinator_scheduling_environment::Asset;
use ordinator_scheduling_environment::work_order::WorkOrderNumber;
use ordinator_scheduling_environment::work_order::operation::ActivityNumber;
use ordinator_scheduling_environment::work_order::work_order_analytic::status_codes::MaterialStatus;
use ordinator_scheduling_environment::worker_environment::resources::ActorCompositeId;
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
pub mod auth;
// This is a DTO object, it should be moved out of the
// `scheduling-environment`
#[derive(
    Hash, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, ToSchema, TS,
)]
#[ts(export, export_to = "../../../ordinator-frontends/src/types/dto/")]
pub struct AssetNames(String);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, ToSchema, TS)]
#[ts(export, export_to = "../../../ordinator-frontends/src/types/dto/")]
pub struct PeriodDto(pub String);

#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, ToSchema, TS, Default,
)]
#[ts(export, export_to = "../../../ordinator-frontends/src/types/dto/")]
/// Example 2025-02-20
pub struct NaiveDateDto(pub String);

#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, ToSchema, TS, Default,
)]
#[ts(export, export_to = "../../../ordinator-frontends/src/types/dto/")]
/// Example 2025-02-20T07:00:00
pub struct DateTimeDto(pub String);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, ToSchema, TS)]
#[ts(export, export_to = "../../../ordinator-frontends/src/types/dto/")]
pub struct WorkOrderNumberDto(pub u64);

#[derive(Debug, ToSchema, Serialize, TS)]
#[ts(export, export_to = "../../../ordinator-frontends/src/types/dto/")]
pub enum MaterialStatusDto
{
    Smat,
    Nmat,
    Cmat,
    Wmat,
    Pmat,
    Unknown,
}

impl From<MaterialStatus> for MaterialStatusDto
{
    fn from(value: MaterialStatus) -> Self
    {
        match value {
            MaterialStatus::Smat => MaterialStatusDto::Smat,
            MaterialStatus::Nmat => MaterialStatusDto::Nmat,
            MaterialStatus::Cmat => MaterialStatusDto::Cmat,
            MaterialStatus::Wmat => MaterialStatusDto::Wmat,
            MaterialStatus::Pmat => MaterialStatusDto::Pmat,
            MaterialStatus::Unknown => MaterialStatusDto::Unknown,
        }
    }
}

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
        let naive_date = NaiveDate::parse_from_str(&value.0, "%Y-%m-%d")?;
        Ok(naive_date)
    }
}

// ISSUE #000 How do we handle datetime?
impl TryFrom<DateTimeDto> for DateTime<Utc>
{
    type Error = chrono::ParseError;

    fn try_from(value: DateTimeDto) -> Result<Self, Self::Error>
    {
        let dt = DateTime::parse_from_rfc3339(&value.0)?;
        Ok(dt.to_utc())
    }
}

pub type TotalSystemSolution =
    SystemSolution<StrategicSolution, TacticalSolution, SupervisorSolution, OperationalSolution>;

#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, ToSchema, Serialize, TS)]
#[ts(export, export_to = "../../../ordinator-frontends/src/types/dto/")]
pub struct IdStringDto(String);

impl IdStringDto
{
    pub fn new(id: String) -> Self
    {
        Self(id)
    }
}

impl From<ActorCompositeId> for IdStringDto
{
    fn from(value: ActorCompositeId) -> Self
    {
        Self(value.0)
    }
}

#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, ToSchema, Serialize, TS)]
#[ts(export, export_to = "../../../ordinator-frontends/src/types/dto/")]
pub struct IdDto
{
    id: String,
    resources: Vec<String>,
    asset: Vec<AssetNames>,
}

impl From<ActorCompositeId> for IdDto
{
    fn from(value: ActorCompositeId) -> Self
    {
        Self {
            id: value.0,
            resources: value.1.iter().map(|e| e.to_string()).collect(),
            asset: value
                .2
                .assets()
                .iter()
                .map(|e| AssetNames(e.to_string()))
                .collect(),
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
