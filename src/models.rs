mod user;
mod server;
mod test;
mod data_point;
mod schedule_entry;

use std::fmt::{Display, Formatter};
use std::str::FromStr;
use crate::my_vector::MyVector;

pub struct SiteData { // Used to package all site data together for ease of use
    pub users: MyVector<User>,
    pub servers: MyVector<Server>,
}

#[derive(Clone)]
pub struct User {
    username: String, // Primary key
    forename: Option<String>,
    surname: Option<String>,
    position: Position, // Access levels not yet implemented
}

#[derive(Clone)] // Can copy the position
pub enum Position { // The positions that a user can have
    Developer,
    Manager,
}

impl Display for Position { // Allows to_string to be ran
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Position::Developer => write!(f, "{}", "Developer"),
            Position::Manager => write!(f, "{}", "Manager")
        }
    }
}

impl FromStr for Position {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s { 
            "Manager" => Ok(Self::Manager),
            _ => Ok(Self::Developer),
        }
    }
}

#[derive(Clone)]
pub struct Server {
    id: String, // Primary key
    name: String,
    created_by: String, // Foreign key
    ram: u32, // In MB
    cpu: u32, // Number of cores
    pub tests: MyVector<Test>,
}

#[derive(Clone)]
pub struct Test {
    id: String, // Primary key
    pub data: MyVector<DataPoint>,
}

#[derive(Clone)]
pub struct DataPoint {
    time: String,
    ram: u32,
    cpu: u32,
    comment: Option<String>,
}

#[derive(Clone)]
pub struct ScheduleEntry {
    id: String, // Primary Key
    datetime: String,
    assignees: MyVector<String>,
    test: Test,
}