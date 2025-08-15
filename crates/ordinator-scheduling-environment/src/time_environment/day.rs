use chrono::NaiveDate;
use rust_xlsxwriter::IntoExcelData;
use serde::Deserialize;
use serde::Serialize;

use crate::work_order::operation::Work;

// NOTE
// You will need the [`TimeEnvironment`] to find the correct day, the
// index here relies on the `time_environment` to find the correct date.
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
pub struct Day(pub NaiveDate);

impl Day
{
    pub fn range(start_day: Day, range: u64) -> Vec<Day>
    {
        (0..range)
            .map(|e| {
                Day(start_day
                    .0
                    .checked_add_days(chrono::Days::new(e))
                    .expect("NaiveDate exceeded its range"))
            })
            .collect()
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
