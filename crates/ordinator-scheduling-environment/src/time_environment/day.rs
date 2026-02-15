use std::fmt::Display;
use std::fmt::{self};

use chrono::NaiveDate;
use rust_xlsxwriter::IntoExcelData;
use serde::Deserialize;
use serde::Serialize;

use crate::work_order::operation::Work;

// Note: The day index relies on `TimeEnvironment` to resolve the correct date
#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
pub struct Days
{
    // #[serde(with = "any_key_map")]
    pub days: Vec<Work>,
}

impl Days
{
    pub fn new(days: Vec<Work>) -> Self
    {
        Self { days }
    }

    pub fn zero_from_existing(days: &Days) -> Self
    {
        let mut value = days.days.to_vec();
        value.fill(Work::from(0.0));
        Self { days: value }
    }
}

#[derive(Eq, PartialEq, Hash, Clone, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub struct Day
{
    pub day_index: usize,
    pub date: NaiveDate,
}

impl Day
{
    pub fn new(day_index: usize, date: NaiveDate) -> Self
    {
        Day { day_index, date }
    }
}

impl Display for Day
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        write!(f, "{}", self.date)
    }
}

#[derive(Debug, Clone)]
pub struct OptionDay(pub Option<NaiveDate>);

impl IntoExcelData for OptionDay
{
    fn write(
        self,
        worksheet: &mut rust_xlsxwriter::Worksheet,
        row: rust_xlsxwriter::RowNum,
        col: rust_xlsxwriter::ColNum,
    ) -> Result<&mut rust_xlsxwriter::Worksheet, rust_xlsxwriter::XlsxError>
    {
        let value = match self.0 {
            Some(day) => day.to_string(),
            None => "".to_string(),
        };

        worksheet.write_string(row, col, value)
    }

    fn write_with_format<'a>(
        self,
        worksheet: &'a mut rust_xlsxwriter::Worksheet,
        row: rust_xlsxwriter::RowNum,
        col: rust_xlsxwriter::ColNum,
        format: &rust_xlsxwriter::Format,
    ) -> Result<&'a mut rust_xlsxwriter::Worksheet, rust_xlsxwriter::XlsxError>
    {
        let value = match self.0 {
            Some(day) => day.to_string(),
            None => "".to_string(),
        };

        worksheet.write_string_with_format(row, col, value, format)
    }
}
