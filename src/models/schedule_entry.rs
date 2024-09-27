use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use crate::models::ScheduleEntry;
use crate::my_vector::SafePointer;

impl ScheduleEntry {
    pub fn new(id: String, datetime: String, assignees: String, test: String) -> Self {
        Self {
            id,
            datetime,
            assignees,
            test,
        }
    }

    pub fn get_id(&self) -> String {
        self.id.clone()
    }

    pub fn set_id(&mut self, id: String) {
        self.id = id;
    }

    pub fn get_datetime(&self) -> String {
        self.datetime.clone()
    }

    pub fn set_datetime(&mut self, datetime: String) {
        self.datetime = datetime;
    }

    pub fn get_assignees(&self) -> String {
        self.assignees.clone()
    }

    pub fn set_assignees(&mut self, assignees: String) {
        self.assignees = assignees;
    }

    pub fn get_test(&self) -> String {
        self.test.clone()
    }

    pub fn set_test(&mut self, test: String) {
        self.test = test;
    }
}

impl Display for ScheduleEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let formatted = format!(
            "{},{},{},{}",
            self.id,
            self.datetime,
            self.assignees,
            self.test,
        );
        write!(f, "{}", formatted)
    }
}

impl FromStr for ScheduleEntry {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(',').collect();
        if parts.len() != 4 {
            return Err("Invalid string format: expected 4 parts separated by commas.".to_string());
        }

        Ok(Self {
            id: parts[0].to_string(),
            datetime: parts[1].to_string(),
            assignees: parts[2].to_string(),
            test: parts[3].to_string(),
        })
    }
}

impl PartialEq<Self> for ScheduleEntry {
    fn eq(&self, other: &Self) -> bool {
        self.to_string() == other.to_string()
    }
}

impl PartialOrd for ScheduleEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.datetime.partial_cmp(&other.datetime)
    }
}

unsafe impl Send for SafePointer<ScheduleEntry> {}
unsafe impl Sync for SafePointer<ScheduleEntry> {}
