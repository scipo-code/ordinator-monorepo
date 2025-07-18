use std::fmt::Display;

use rust_xlsxwriter::IntoExcelData;
use serde::Deserialize;
use serde::Serialize;

use crate::Asset;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FunctionalLocation
{
    pub string: String,
    pub asset: Asset,
}

impl FunctionalLocation
{
    pub fn new(functional_location: &str) -> Self
    {
        let asset_string = functional_location
            .split('/')
            .next()
            .expect("All work orders need to have an Asset")
            .trim();

        let asset = Asset::new_from_string(asset_string).unwrap_or(Asset::Unknown);
        Self {
            string: functional_location.to_string(),
            asset,
        }
    }

    pub fn sector(&self) -> Option<&str>
    {
        self.string.split('/').nth(1).map(|str| str.trim())
    }

    pub fn system(&self) -> Option<char>
    {
        self.string
            .split('/')
            .nth(2)
            .map(|str| str.trim())?
            .chars()
            .nth(0)
    }

    pub fn subsystem(&self) -> Option<char>
    {
        self.string
            .split('/')
            .nth(2)
            .map(|str| str.trim())?
            .chars()
            .nth(1)
    }

    pub fn equipment_tag(&self) -> Option<&str>
    {
        self.string.split('/').nth(3).map(|str| str.trim())
    }
}

impl Default for FunctionalLocation
{
    fn default() -> Self
    {
        FunctionalLocation {
            string: "Unknown".to_string(),
            asset: Asset::Unknown,
        }
    }
}

impl Display for FunctionalLocation
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        write!(f, "{}", self.string)
    }
}

impl IntoExcelData for FunctionalLocation
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
    use super::FunctionalLocation;
    use crate::Asset;

    #[test]
    fn test_functional_location_string_parsing()
    {
        let functional_location_string_0 = "DF/A/09/RA-REA";
        let functional_location_0 = FunctionalLocation::new(functional_location_string_0);
        assert_eq!(functional_location_0.asset, Asset::DF);
        assert_eq!(functional_location_0.sector(), Some("A"));
        assert_eq!(functional_location_0.system(), Some('0'));
        assert_eq!(functional_location_0.subsystem(), Some('9'));
        assert_eq!(functional_location_0.equipment_tag(), Some("RA-REA"));

        let functional_location_string_1 = " DF  / A / 09 /  RA-REA";
        let functional_location_1 = FunctionalLocation::new(functional_location_string_1);
        assert_eq!(functional_location_1.asset, Asset::DF);
        assert_eq!(functional_location_1.sector(), Some("A"));
        assert_eq!(functional_location_1.system(), Some('0'));
        assert_eq!(functional_location_1.subsystem(), Some('9'));
        assert_eq!(functional_location_1.equipment_tag(), Some("RA-REA"));

        let functional_location_string_2 = "TEST  / A / 09 /  RA-REA";
        let functional_location_2 = FunctionalLocation::new(functional_location_string_2);
        dbg!(&functional_location_2);

        assert_eq!(functional_location_2.asset, Asset::Test);
        assert_eq!(functional_location_2.sector(), Some("A"));
        assert_eq!(functional_location_2.system(), Some('0'));
        assert_eq!(functional_location_2.subsystem(), Some('9'));
        assert_eq!(functional_location_2.equipment_tag(), Some("RA-REA"));
    }
}
