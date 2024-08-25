use crate::models::{ScheduleEntry, Test};
use crate::my_vector::MyVector;

impl ScheduleEntry {
    pub fn new(id: String, datetime: String, test: Test) -> Self {
        Self {
            id,
            datetime,
            assignees: MyVector::new(),
            test,
        }
    }
}