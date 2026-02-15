use std::fmt::Display;
use std::str::FromStr;

use chrono::NaiveTime;
use colored::*;
use rust_xlsxwriter::IntoExcelData;
use serde::Deserialize;
use serde::Serialize;
use strum_macros::EnumIter;

use super::availability::Availability;
use crate::Asset;

/// Enum representing all available skills needed to schedule work orders.
#[derive(
    Hash,
    PartialOrd,
    Ord,
    Debug,
    Eq,
    PartialEq,
    Clone,
    Serialize,
    Deserialize,
    EnumIter,
    clap::ValueEnum,
    Copy,
)]
pub enum Skill
{
    #[serde(rename = "MTN-MECH")]
    MtnMech,
    #[serde(rename = "MTN-ELEC")]
    MtnElec,
    #[serde(rename = "MTN-INST")]
    MtnInst,
    #[serde(rename = "MTN-ROUS")]
    MtnRous,
    #[serde(rename = "MTN-RIGG")]
    MtnRigg,
    #[serde(rename = "MTN-SCAF")]
    MtnScaf,
    #[serde(rename = "MTN-PIPF")]
    MtnPipf,
    #[serde(rename = "MTN-CRAN")]
    MtnCran,
    #[serde(rename = "MTN-ROPE")]
    MtnRope,
    #[serde(rename = "MTN-PAIN")]
    MtnPain,
    #[serde(rename = "MTN-TELE")]
    MtnTele,
    #[serde(rename = "MTN-TURB")]
    MtnTurb,
    #[serde(rename = "MTN-LAGG")]
    MtnLagg,
    #[serde(rename = "MTN-SAT")]
    MtnSat,
    #[serde(rename = "PRODTECH")]
    Prodtech,
    #[serde(rename = "PRODLABO")]
    Prodlabo,
    #[serde(rename = "VEN-INST")]
    VenInst,
    #[serde(rename = "VEN-ACCO")]
    VenAcco,
    #[serde(rename = "VEN-COMM")]
    VenComm,
    #[serde(rename = "VEN-CRAN")]
    VenCran,
    #[serde(rename = "VEN-ELEC")]
    VenElec,
    #[serde(rename = "VEN-HVAC")]
    VenHvac,
    #[serde(rename = "VEN-MECH")]
    VenMech,
    #[serde(rename = "VEN-METE")]
    VenMete,
    #[serde(rename = "VEN-SUBS")]
    VenSubs,
    #[serde(rename = "VEN-ROPE")]
    VenRope,
    #[serde(rename = "VEN-FFEQ")]
    VenFfeq,
    #[serde(rename = "VEN-TURB")]
    VenTurb,
    #[serde(rename = "VEN-SCAF")]
    VenScaf,
    #[serde(rename = "VEN-INSP")]
    VenInsp,
    #[serde(rename = "INP-SITE")]
    InpSite,
    #[serde(rename = "MAINONSH")]
    Mainonsh,
    #[serde(rename = "DRILLING")]
    Drilling,
    #[serde(rename = "WELLMAIN")]
    Wellmain,
    #[serde(rename = "WELLSUPV")]
    Wellsupv,
    #[serde(rename = "WELLTECH")]
    Welltech,
    #[serde(rename = "MEDIC")]
    Medic,
    #[serde(rename = "QAQCELEC")]
    Qaqcelec,
    #[serde(rename = "QAQCMECH")]
    Qaqcmech,
    #[serde(rename = "QAQCPAIN")]
    Qaqcpain,
    #[serde(rename = "PRODCCR")]
    Prodccr,
    #[serde(rename = "CMP-RIGG")]
    CmpRigg,
    #[serde(rename = "CMP-SCAF")]
    CmpScaf,
    #[serde(rename = "CON-NPT")]
    ConNpt,
    #[serde(rename = "CON-VEN")]
    ConVen,
    #[serde(rename = "CON-ELEC")]
    ConElec,
    #[serde(rename = "CON-INPF")]
    ConInpf,
    #[serde(rename = "CON-INST")]
    ConInst,
    #[serde(rename = "CON-LAGG")]
    ConLagg,
    #[serde(rename = "CON-NDTI")]
    ConNdti,
    #[serde(rename = "CON-SCAF")]
    ConScaf,
    #[serde(rename = "CON-PAIN")]
    ConPain,
    #[serde(rename = "CON-RIGG")]
    ConRigg,
    #[serde(rename = "CON-ROPE")]
    ConRope,
    #[serde(rename = "CON-WELD")]
    ConWeld,
}

impl FromStr for Skill
{
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err>
    {
        let resource = match s {
            "MTN-PIPF" => Skill::MtnPipf,
            "VEN-TURB" => Skill::VenTurb,
            "CON-VEN" => Skill::ConVen,
            "MTN-LAGG" => Skill::MtnLagg,
            "VEN-SCAF" => Skill::VenScaf,
            "MTN-ROPE" => Skill::MtnRope,
            "VEN-INSP" => Skill::VenInsp,
            "INP-SITE" => Skill::InpSite,
            "VEN-INST" => Skill::VenInst,
            "MAINONSH" => Skill::Mainonsh,
            "DRILLING" => Skill::Drilling,
            "WELLMAIN" => Skill::Wellmain,
            "WELLSUPV" => Skill::Wellsupv,
            "WELLTECH" => Skill::Welltech,
            "CON-ELEC" => Skill::ConElec,
            "CON-INPF" => Skill::ConInpf,
            "CON-INST" => Skill::ConInst,
            "CON-LAGG" => Skill::ConLagg,
            "CON-NDTI" => Skill::ConNdti,
            "CON-SCAF" => Skill::ConScaf,
            "CON-PAIN" => Skill::ConPain,
            "CON-RIGG" => Skill::ConRigg,
            "CON-ROPE" => Skill::ConRope,
            "CON-WELD" => Skill::ConWeld,
            "MTN-ROUS" => Skill::MtnRous,
            "MTN-CRAN" => Skill::MtnCran,
            "MTN-ELEC" => Skill::MtnElec,
            "MTN-INST" => Skill::MtnInst,
            "MTN-MECH" => Skill::MtnMech,
            "MTN-RIGG" => Skill::MtnRigg,
            "MTN-SCAF" => Skill::MtnScaf,
            "MTN-PAIN" => Skill::MtnPain,
            "MTN-TELE" => Skill::MtnTele,
            "MTN-TURB" => Skill::MtnTurb,
            "MEDIC" => Skill::Medic,
            "PRODLABO" => Skill::Prodlabo,
            "PRODTECH" => Skill::Prodtech,
            "MTN-SAT" => Skill::MtnSat,
            "VEN-ACCO" => Skill::VenAcco,
            "VEN-COMM" => Skill::VenComm,
            "VEN-CRAN" => Skill::VenCran,
            "VEN-ELEC" => Skill::VenElec,
            "VEN-HVAC" => Skill::VenHvac,
            "VEN-MECH" => Skill::VenMech,
            "VEN-METE" => Skill::VenMete,
            "VEN-SUBS" => Skill::VenSubs,
            "VEN-ROPE" => Skill::VenRope,
            "QAQCELEC" => Skill::Qaqcelec,
            "QAQCMECH" => Skill::Qaqcmech,
            "QAQCPAIN" => Skill::Qaqcpain,
            "PRODCCR" => Skill::Prodccr,
            "VEN-FFEQ" => Skill::VenFfeq,
            "CMP-RIGG" => Skill::CmpRigg,
            "CMP-SCAF" => Skill::CmpScaf,
            "CON-NPT" => Skill::ConNpt,
            unknown => return Err(format!("Could not parse Resource: {unknown}")),
        };
        Ok(resource)
    }
}

impl Display for Skill
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        let value = match self {
            Skill::Medic => "MEDIC",
            Skill::MtnCran => "MTN-CRAN",
            Skill::MtnElec => "MTN-ELEC",
            Skill::MtnInst => "MTN-INST",
            Skill::MtnLagg => "MTN-LAGG",
            Skill::MtnMech => "MTN-MECH",
            Skill::MtnPain => "MTN-PAIN",
            Skill::MtnPipf => "MTN-PIPF",
            Skill::MtnRigg => "MTN-RIGG",
            Skill::MtnRope => "MTN-ROPE",
            Skill::MtnRous => "MTN-ROUS",
            Skill::MtnSat => "MTN-SAT",
            Skill::MtnScaf => "MTN-SCAF",
            Skill::MtnTele => "MTN-TELE",
            Skill::MtnTurb => "MTN-TURB",
            Skill::InpSite => "INP-SITE",
            Skill::Prodlabo => "PRODLABO",
            Skill::Prodtech => "PRODTECH",
            Skill::VenAcco => "VEN-ACCO",
            Skill::VenComm => "VEN-COMM",
            Skill::VenCran => "VEN-CRAN",
            Skill::VenElec => "VEN-ELEC",
            Skill::VenHvac => "VEN-HVAC",
            Skill::VenInsp => "VEN-INSP",
            Skill::VenInst => "VEN-INST",
            Skill::VenMech => "VEN-MECH",
            Skill::VenMete => "VEN-METE",
            Skill::VenRope => "VEN-ROPE",
            Skill::VenScaf => "VEN-SCAF",
            Skill::VenSubs => "VEN-SUBS",
            Skill::Qaqcelec => "QAQCELEC",
            Skill::Qaqcmech => "QAQCMECH",
            Skill::Qaqcpain => "QAQCPAIN",
            Skill::Wellsupv => "WELLSUPV",
            Skill::VenTurb => "VEN-TURB",
            Skill::ConVen => "CON-VEN",
            Skill::Mainonsh => "MAINONSH",
            Skill::Drilling => "DRILLING",
            Skill::Wellmain => "WELLMAIN",
            Skill::Welltech => "WELLTECH",
            Skill::ConElec => "CON-ELEC",
            Skill::ConInpf => "CON-INPF",
            Skill::ConInst => "CON-INST",
            Skill::ConLagg => "CON-LAGG",
            Skill::ConNdti => "CON-NDTI",
            Skill::ConScaf => "CON-SCAF",
            Skill::ConPain => "CON-PAIN",
            Skill::ConRigg => "CON-RIGG",
            Skill::ConRope => "CON-ROPE",
            Skill::ConWeld => "CON-WELD",
            Skill::Prodccr => "PRODCCR",
            Skill::VenFfeq => "VEN-FFEQ",
            Skill::CmpRigg => "CMP-RIGG",
            Skill::CmpScaf => "CMP-SCAF",
            Skill::ConNpt => "CON-NPT",
        };
        write!(f, "{}", value)
    }
}
impl Skill
{
    pub fn is_ven_variant(&self) -> bool
    {
        matches!(
            self,
            Skill::VenAcco
                | Skill::VenComm
                | Skill::VenCran
                | Skill::VenElec
                | Skill::VenHvac
                | Skill::VenInsp
                | Skill::VenInst
                | Skill::VenMech
                | Skill::VenMete
                | Skill::VenRope
                | Skill::VenScaf
                | Skill::VenSubs
        )
    }

    pub fn is_fmc(&self) -> bool
    {
        matches!(
            self,
            Self::MtnRope
                | Self::MtnScaf
                | Self::MtnRigg
                | Self::MtnLagg
                | Self::MtnPipf
                | Self::MtnPain
        )
    }
}

// TODO: Add function to integrate availability into the composite ID
#[derive(Eq, Hash, Ord, PartialOrd, PartialEq, Serialize, Deserialize, Clone, Default)]
pub struct ActorCompositeId(pub String, pub Vec<Skill>, pub Availability);

// Custom Debug implementation for colored terminal output
impl std::fmt::Debug for ActorCompositeId
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        write!(
            f,
            "{}",
            format!(
                "Id: Id({}, resources: {:?}, assets: {:?})",
                self.0, self.1, self.2
            )
            .blue(),
        )
    }
}

impl ActorCompositeId
{
    pub fn new(id_employee: &str, resources: Vec<Skill>, availability: Availability) -> Self
    {
        ActorCompositeId(id_employee.to_string(), resources, availability)
    }

    pub fn asset(&self) -> &Asset
    {
        self.2.main_asset()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, EnumIter, clap::ValueEnum)]
pub enum Shift
{
    Day,
    Night,
}

impl Shift
{
    pub fn generate_time_intervals(&self) -> (NaiveTime, NaiveTime)
    {
        match self {
            Shift::Day => (
                NaiveTime::from_hms_opt(7, 0, 0).unwrap(),
                NaiveTime::from_hms_opt(19, 0, 0).unwrap(),
            ),
            Shift::Night => (
                NaiveTime::from_hms_opt(19, 0, 0).unwrap(),
                NaiveTime::from_hms_opt(7, 0, 0).unwrap(),
            ),
        }
    }
}

// Note: Only Debug::alternate() formatting supports colored output
impl Display for ActorCompositeId
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        write!(
            f,
            "Id: {}\nresources: {:?}\navailability: {:?}",
            self.0, self.1, self.2
        )
    }
}

impl IntoExcelData for Skill
{
    fn write(
        self,
        worksheet: &mut rust_xlsxwriter::Worksheet,
        row: rust_xlsxwriter::RowNum,
        col: rust_xlsxwriter::ColNum,
    ) -> Result<&mut rust_xlsxwriter::Worksheet, rust_xlsxwriter::XlsxError>
    {
        let value = self.to_string();
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
        let value = self.to_string();
        worksheet.write_string_with_format(row, col, value, format)
    }
}
