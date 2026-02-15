use anyhow::bail;
use ordinator_scheduling_environment::work_order::operation::Operation;

#[derive(Clone, Default, Hash, Eq, PartialEq, PartialOrd, Ord, Debug)]
pub enum Delegate
{
    #[default]
    Assess,
    Assign,
    Unassign,
    Drop,
    Done,
    Fixed,
}

impl Delegate
{
    pub fn build(operation: &Operation) -> Delegate
    {
        if operation.work_remaining().is_zero() {
            return Delegate::Done;
        }
        Delegate::Assess
    }

    pub fn is_done(&self) -> bool
    {
        matches!(self, Self::Done)
    }

    pub fn is_assign(&self) -> bool
    {
        matches!(self, Self::Assign)
    }

    pub fn is_assess(&self) -> bool
    {
        matches!(self, Self::Assess)
    }

    pub fn is_drop(&self) -> bool
    {
        matches!(self, Self::Drop)
    }

    pub fn state_change_to_unassign(&mut self) -> anyhow::Result<()>
    {
        match self {
            Delegate::Assess => {
                *self = Delegate::Unassign;
                Ok(())
            }
            Delegate::Assign => {
                bail!("Only Delegate::Assess work_order_activities can have their state changed")
            }
            Delegate::Unassign => {
                bail!("Only Delegate::Assess work_order_activities can have their state changed")
            }
            Delegate::Drop => {
                bail!("Only Delegate::Assess work_order_activities can have their state changed")
            }
            Delegate::Done => {
                bail!("Only Delegate::Assess work_order_activities can have their state changed")
            }
            Delegate::Fixed => {
                bail!("Only Delegate::Assess work_order_activities can have their state changed")
            }
        }
    }

    pub fn state_change_to_assign(&mut self) -> anyhow::Result<()>
    {
        match self {
            Delegate::Assess => {
                *self = Delegate::Assign;
                Ok(())
            }
            Delegate::Assign => {
                bail!("Only Delegate::Assess work_order_activities can have their state changed")
            }
            Delegate::Unassign => {
                bail!("Only Delegate::Assess work_order_activities can have their state changed")
            }
            Delegate::Drop => {
                bail!("Only Delegate::Assess work_order_activities can have their state changed")
            }
            Delegate::Done => {
                bail!("Only Delegate::Assess work_order_activities can have their state changed")
            }
            Delegate::Fixed => {
                bail!("Only Delegate::Assess work_order_activities can have their state changed")
            }
        }
    }
}
