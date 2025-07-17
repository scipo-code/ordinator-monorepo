use chrono::Datelike;
use clap::Args;
use rust_xlsxwriter::IntoExcelData;
use serde::Deserialize;
use serde::Serialize;

// ISSUE #000 TODO [ ] 2025-07-17 This domain type should be extended
// significantly. Interview Brian
#[derive(Default, Args, Clone, Serialize, Deserialize, Debug)]
pub struct UnloadingPoint
{
    pub string: String,
    pub period_string: Option<String>,
}
// This field simply needs to be derived when we want a period. That is the only
// way of implementing it in a way that will scale into the future. You cannot
// afford not to FIX things that are in error.
// You have to make this one work correctly,
impl UnloadingPoint
{
    // UnloadingPoint is a value object. That means that it should be always built
    // from new.
    // You are doing it wrong again. You should split the original. You are
    // performing duplication of data.
    pub fn new(unloading_point_string: String, period_string: Option<String>) -> Self
    {
        if let Some(period_string) = period_string {
            return Self {
                string: unloading_point_string,
                period_string: Some(period_string),
            };
        }
        // let re = regex::Regex::new(r"(\d{2})?-?[W|w](\d+)-?[W|w]?(\d+)").unwrap();
        let regex = regex::Regex::new(
            r"(?P<year>\d{4}|\d{2})?-?[Ww](?P<from>\d{1,2})-?[Ww]?(?P<to>\d{1,2})?",
        )
        .unwrap();

        let period_string = match regex.captures(&unloading_point_string) {
            Some(caps) => {
                let year = caps
                    .name("year")
                    .map(|m| m.as_str())
                    .map(|y| match y.len() {
                        2 => {
                            2000 + y
                                .parse::<u32>()
                                .expect("RegEx is hard coded this should not be able to fail.")
                        }
                        4 => y
                            .parse::<u32>()
                            .expect("RegEx is hard coded this should not be able to fail."),
                        _ => 2025,
                    })
                    .unwrap_or(chrono::Utc::now().year() as u32);

                let from = caps
                    .name("from")
                    .expect("This value is hardcoded")
                    .as_str()
                    .parse::<u32>()
                    .expect("This value is hardcoded");

                let to = match caps.name("to") {
                    Some(regex_match) => regex_match
                        .as_str()
                        .parse::<u32>()
                        .expect("This value is hardcoded"),
                    None => from + 1,
                };

                Some(year.to_string() + "-W" + &from.to_string() + "-W" + &to.to_string())
            }
            None => None,
        };

        UnloadingPoint {
            string: unloading_point_string.clone(),
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

#[cfg(test)]
mod tests
{
    use crate::work_order::work_order_dates::unloading_point::UnloadingPoint;

    #[test]
    fn test_regex_unloading_point()
    {
        let period_1 = "2025-W01-W02";
        let period_2 = "1994-W01-W02";
        let period_3 = "86-W3-4";

        let period_4 = "25-W5-06";
        let period_5 = "W1-2";
        let period_6 = "W9-10";

        let period_7 = "2025-w01-w02";
        let period_8 = "1994-w01-w02";
        let period_9 = "86-w3-4";

        let period_10 = "25-w5-06";
        let period_11 = "w1-2";
        let period_12 = "w9-10";

        let period_13 = "2025-9-10";
        let period_14 = "2025W10W11";
        let period_15 = "2025W10-13";

        let capture_1 = UnloadingPoint::new(period_1.to_string(), None);
        let capture_2 = UnloadingPoint::new(period_2.to_string(), None);
        let capture_3 = UnloadingPoint::new(period_3.to_string(), None);
        let capture_4 = UnloadingPoint::new(period_4.to_string(), None);
        let capture_5 = UnloadingPoint::new(period_5.to_string(), None);
        let capture_6 = UnloadingPoint::new(period_6.to_string(), None);
        let capture_7 = UnloadingPoint::new(period_7.to_string(), None);
        let capture_8 = UnloadingPoint::new(period_8.to_string(), None);
        let capture_9 = UnloadingPoint::new(period_9.to_string(), None);
        let capture_10 = UnloadingPoint::new(period_10.to_string(), None);
        let capture_11 = UnloadingPoint::new(period_11.to_string(), None);
        let capture_12 = UnloadingPoint::new(period_12.to_string(), None);
        let capture_13 = UnloadingPoint::new(period_13.to_string(), None);
        let capture_14 = UnloadingPoint::new(period_14.to_string(), None);
        let capture_15 = UnloadingPoint::new(period_15.to_string(), None);

        assert_eq!(capture_1.period_string, Some("2025-W1-W2".to_string()));
        assert_eq!(capture_2.period_string, Some("1994-W1-W2".to_string()));
        assert_eq!(capture_3.period_string, Some("2086-W3-W4".to_string()));

        assert_eq!(capture_4.period_string, Some("2025-W5-W6".to_string()));
        assert_eq!(capture_5.period_string, Some("2025-W1-W2".to_string()));
        assert_eq!(capture_6.period_string, Some("2025-W9-W10".to_string()));

        assert_eq!(capture_7.period_string, Some("2025-W1-W2".to_string()));
        assert_eq!(capture_8.period_string, Some("1994-W1-W2".to_string()));
        assert_eq!(capture_9.period_string, Some("2086-W3-W4".to_string()));

        assert_eq!(capture_10.period_string, Some("2025-W5-W6".to_string()));
        assert_eq!(capture_11.period_string, Some("2025-W1-W2".to_string()));
        assert_eq!(capture_12.period_string, Some("2025-W9-W10".to_string()));

        assert_eq!(capture_13.period_string, None);
        assert_eq!(capture_14.period_string, Some("2025-W10-W11".to_string()));
        assert_eq!(capture_15.period_string, Some("2025-W10-W13".to_string()));
    }
}
