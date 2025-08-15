use serde::Deserialize;
use serde::Serialize;

#[derive(Debug)]
pub struct MaterialRepo
{
    pub material_to_period: MaterialToPeriod,
    // ISSUE #000 [ ] - add control tower or other data source to the program
    // control_tower: ControlTower,
}

impl MaterialRepo
{
    pub fn new(material_to_period: MaterialToPeriod) -> Self
    {
        Self { material_to_period }
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
