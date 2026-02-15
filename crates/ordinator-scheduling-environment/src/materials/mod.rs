use serde::Deserialize;
use serde::Serialize;

#[derive(Debug)]
pub struct MaterialRepo
{
    pub material_to_period: MaterialToPeriod,
    // TODO: Add control tower or other data source to the program
    // control_tower: ControlTower,
}

pub struct MaterialRepoBuilder
{
    material_to_period: Option<MaterialToPeriod>,
}

impl MaterialRepoBuilder
{
    pub fn new() -> Self
    {
        Self {
            material_to_period: None,
        }
    }

    pub fn material_to_period(mut self, material_to_period: MaterialToPeriod) -> Self
    {
        self.material_to_period = Some(material_to_period);
        self
    }

    pub fn build(self) -> MaterialRepo
    {
        MaterialRepo {
            material_to_period: self.material_to_period.unwrap_or_else(|| {
                MaterialToPeriod::builder().build()
            }),
        }
    }
}

impl MaterialRepo
{
    pub fn new(material_to_period: MaterialToPeriod) -> Self
    {
        Self { material_to_period }
    }

    pub fn builder() -> MaterialRepoBuilder
    {
        MaterialRepoBuilder::new()
    }
}
#[derive(Eq, PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct MaterialToPeriod
{
    pub nmat: usize,
    pub smat: usize,
    pub cmat: usize,
    pub pmat: usize,
    pub wmat: usize,
}

pub struct MaterialToPeriodBuilder
{
    nmat: Option<usize>,
    smat: Option<usize>,
    cmat: Option<usize>,
    pmat: Option<usize>,
    wmat: Option<usize>,
}

impl MaterialToPeriodBuilder
{
    pub fn new() -> Self
    {
        Self {
            nmat: None,
            smat: None,
            cmat: None,
            pmat: None,
            wmat: None,
        }
    }

    pub fn nmat(mut self, nmat: usize) -> Self
    {
        self.nmat = Some(nmat);
        self
    }

    pub fn smat(mut self, smat: usize) -> Self
    {
        self.smat = Some(smat);
        self
    }

    pub fn cmat(mut self, cmat: usize) -> Self
    {
        self.cmat = Some(cmat);
        self
    }

    pub fn pmat(mut self, pmat: usize) -> Self
    {
        self.pmat = Some(pmat);
        self
    }

    pub fn wmat(mut self, wmat: usize) -> Self
    {
        self.wmat = Some(wmat);
        self
    }

    pub fn build(self) -> MaterialToPeriod
    {
        MaterialToPeriod {
            nmat: self.nmat.unwrap_or(0),
            smat: self.smat.unwrap_or(0),
            cmat: self.cmat.unwrap_or(2),
            pmat: self.pmat.unwrap_or(3),
            wmat: self.wmat.unwrap_or(3),
        }
    }
}

impl MaterialToPeriod
{
    pub fn builder() -> MaterialToPeriodBuilder
    {
        MaterialToPeriodBuilder::new()
    }
}
