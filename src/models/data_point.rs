use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use crate::models::DataPoint;
use crate::my_vector::SafePointer;

impl DataPoint {
    pub fn new(time: String, ram: u32, cpu: u32) -> Self {
        Self{
            time,
            ram,
            cpu,
            comment: None,
        }
    }

    pub fn add_comment(&mut self, comment: String) {
        self.comment = Some(comment)
    }

    pub fn set_time(&mut self, time: String) {
        self.time = time
    }

    pub fn set_ram(&mut self, ram: u32) {
        self.ram = ram
    }

    pub fn set_cpu(&mut self, cpu: u32) {
        self.cpu = cpu
    }

    pub fn get_time(&self) -> String {
        self.time.clone()
    }

    pub fn get_ram(&self) -> u32 {
        self.ram
    }

    pub fn get_cpu(&self) -> u32 {
        self.cpu
    }

    pub fn get_comment(&self) -> Option<String> {
        self.comment.clone()
    }

    pub fn set_comment(&mut self, comment: Option<String>) {
        self.comment = comment;
    }
}

impl Display for DataPoint {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = format!("{},{},{},{}",
            self.time,
            self.ram,
            self.cpu,
            self.comment.clone().unwrap_or_default(),
        );

        write!(f, "{}", str)
    }
}

impl FromStr for DataPoint {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(',').collect();
        if parts.len() != 4 {
            return Err("Invalid string format".to_string())
        }

        Ok(Self {
            time: String::from_str(parts[0]).unwrap_or_default(),
            ram: parts[1].parse::<u32>().unwrap_or_default(),
            cpu: parts[2].parse::<u32>().unwrap_or_default(),
            comment: if !parts[3].is_empty() { Some(String::from_str(parts[3]).unwrap_or_default()) } else { None }
        })
    }
}

impl PartialEq<Self> for DataPoint {
    fn eq(&self, other: &Self) -> bool {
        self.to_string() == other.to_string() // Must match exactly
    }
}

impl PartialOrd for DataPoint {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.cpu.partial_cmp(&other.cpu) // Sort by CPU
    }
}

unsafe impl Send for SafePointer<DataPoint> {}
unsafe impl Sync for SafePointer<DataPoint> {}