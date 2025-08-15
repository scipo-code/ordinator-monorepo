use chrono::DateTime;
use chrono::NaiveDate;
use chrono::TimeDelta;
use chrono::Utc;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::de;

use crate::Asset;

#[derive(Hash, Eq, PartialEq, PartialOrd, Ord, Debug, Serialize, Deserialize, Clone, Default)]
pub struct Availability
{
    #[serde(deserialize_with = "chrono_datetime_deserialize")]
    start_datetime: chrono::DateTime<Utc>,
    #[serde(deserialize_with = "chrono_datetime_deserialize")]
    finish_datetime: chrono::DateTime<Utc>,
    assets: Vec<Asset>,
}

impl Availability
{
    pub fn new(
        start_date: chrono::DateTime<Utc>,
        finish_date: chrono::DateTime<Utc>,
        assets: Vec<Asset>,
    ) -> anyhow::Result<Self>
    {
        if start_date > finish_date {
            return Err(anyhow::anyhow!("Start date later than finish date."));
        }

        Ok(Self {
            start_datetime: start_date,
            finish_datetime: finish_date,
            assets,
        })
    }

    pub fn duration(&self) -> TimeDelta
    {
        self.finish_datetime - self.start_datetime
    }

    pub fn main_asset(&self) -> &Asset
    {
        self.assets.first().expect("This should never happen")
    }

    pub fn assets(&self) -> &Vec<Asset>
    {
        &self.assets
    }

    pub(crate) fn start_date(&self) -> NaiveDate
    {
        self.start_datetime.date_naive()
    }

    pub(crate) fn finish_date(&self) -> NaiveDate
    {
        self.finish_datetime.date_naive()
    }

    pub fn finish_datetime(&self) -> DateTime<Utc>
    {
        self.finish_datetime
    }

    pub fn start_datetime(&self) -> DateTime<Utc>
    {
        self.start_datetime
    }
}

fn chrono_datetime_deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let datetime_str: String = Deserialize::deserialize(deserializer)?;

    let datetime = DateTime::parse_from_rfc3339(&datetime_str).map_err(de::Error::custom)?;
    Ok(datetime.to_utc())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TomlAvailability
{
    start_date: toml::value::Datetime,
    end_date: toml::value::Datetime,
    asset: Vec<Asset>,
}

impl From<TomlAvailability> for Availability
{
    fn from(value: TomlAvailability) -> Self
    {
        let start_date_time: DateTime<Utc> =
            DateTime::parse_from_rfc3339(&value.start_date.to_string())
                .unwrap()
                .to_utc();
        let end_date_time: DateTime<Utc> =
            DateTime::parse_from_rfc3339(&value.end_date.to_string())
                .unwrap()
                .to_utc();

        Self {
            start_datetime: start_date_time,
            finish_datetime: end_date_time,
            assets: value.asset,
        }
    }
}
