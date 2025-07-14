use clap::Args;
use regex::Regex;
use rust_xlsxwriter::IntoExcelData;
use serde::Deserialize;
use serde::Serialize;

#[derive(Default, Args, Clone, Serialize, Deserialize, Debug)]
pub struct UnloadingPoint
{
    pub string: String,
    pub period_string: Option<String>,
}
// This field simply needs to be derived when we want a period. That is the only
// way of implementing it in a way that will scale into the future. You cannot
// afford not to FIX things that are in error.
impl UnloadingPoint
{
    pub fn new(string: String) -> Self
    {
        let week_regex = Regex::new(r"(?P<year>\d{4})-W(?P<from>\d{2})-W(?P<to>\d{2})")
            .expect("This has to produce a vaild Regex");

        let period_string = match week_regex.captures(&string) {
            Some(caps) => {
                let year: u32 = caps["year"].parse().unwrap();
                let from: u32 = caps["from"].parse().unwrap();
                let to: u32 = caps["to"].parse().unwrap();

                Some(year.to_string() + "-W" + &from.to_string() + "-W" + &to.to_string())
            }
            None => None,
        };
        UnloadingPoint {
            string: string.clone(),
            period_string,
        }
    }
}

impl std::fmt::Display for UnloadingPoint
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        write!(f, "{}", self.string)
    }
}

impl IntoExcelData for UnloadingPoint
{
    fn write(
        self,
        worksheet: &mut rust_xlsxwriter::Worksheet,
        row: rust_xlsxwriter::RowNum,
        col: rust_xlsxwriter::ColNum,
    ) -> Result<&mut rust_xlsxwriter::Worksheet, rust_xlsxwriter::XlsxError>
    {
        let value = self.string;
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
        let value = self.string;
        worksheet.write_string_with_format(row, col, value, format)
    }
}
