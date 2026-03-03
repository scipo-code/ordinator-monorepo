use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::HashSet;

use chrono::NaiveDate;
use ordinator_scheduling_environment::time_environment::period::Period;
use ordinator_scheduling_environment::work_order::ActivityRelation;
use ordinator_scheduling_environment::work_order::WorkOrderNumber;
use ordinator_scheduling_environment::work_order::operation::ActivityNumber;
use ordinator_scheduling_environment::work_order::operation::Work;
use ordinator_scheduling_environment::work_order::operation::operation_info::NumberOfPeople;
use ordinator_scheduling_environment::worker_environment::resources::Skill;

use crate::schedule_graph::TechnicianId;

#[derive(Debug)]
pub struct WeeklyView
{
    pub work_orders: HashMap<WorkOrderNumber, WeeklyWorkOrderView>,
    pub periods: Vec<Period>,
    pub skills: HashSet<Skill>,
    pub technicians: HashMap<TechnicianId, TechnicianView>,
}

#[derive(Debug)]
pub struct WeeklyWorkOrderView
{
    pub basic_start_date: Option<NaiveDate>,
    pub latest_allowed_finish_date: NaiveDate,
    pub assigned_period: Option<Period>,
    pub excluded_periods: HashSet<Period>,
    pub activities: Vec<ActivityView>,
}

#[derive(Debug)]
pub struct ActivityView
{
    pub activity_number: ActivityNumber,
    pub number_of_people: NumberOfPeople,
    pub work_remaining: Work,
    pub required_skill: Skill,
    pub relation_to_next: Option<ActivityRelation>,
}

#[derive(Debug)]
pub struct TechnicianView
{
    pub skills: BTreeSet<Skill>,
    pub available_dates: HashSet<NaiveDate>,
}
