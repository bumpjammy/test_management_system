use std::cmp::Ordering;
use std::fmt::Display;
use std::str::FromStr;
use crate::models::{Position, User};
use crate::my_vector::SafePointer;

impl User {
    pub fn new(username: String, position: Position) -> Self {
        Self { // Constructs a new instance of self
            username,
            forename: None, // Default no name
            surname: None,
            position
        }
    }
    
    pub fn get_username(&self) -> String {
        self.username.clone()
    }
    
    pub fn get_forename(&self) -> Option<String> {
        self.forename.clone()
    }
    
    pub fn get_surname(&self) -> Option<String> {
        self.surname.clone()
    }
    
    pub fn get_position(&self) -> Position {
        self.position.clone()
    }
}

impl Display for User { // ToString
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let position = match self.position {
            Position::Developer => "Developer",
            Position::Manager => "Manager"
        }; // Turn the position into a string
        
        let str = format!(
            "{},{},{},{}",
            self.username,
            self.forename.clone().unwrap_or_default(),
            self.surname.clone().unwrap_or_default(),
            position,
        );
        write!(f, "{}", str)
    }
}

impl FromStr for User { // FromString
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(',').collect();
        if parts.len() != 4 {
            return Err("Invalid string format".to_string());
        }

        let position = match parts[3] {
            "Manager" => Position::Manager,
            _ => Position::Developer, // If not a manager, any other values result in a developer
        };

        Ok(User {
            username: parts[0].to_string(),
            forename: if parts[1].is_empty() { None } else { Some(parts[1].to_string()) }, // If nothing there, None
            surname: if parts[2].is_empty() { None } else { Some(parts[2].to_string()) },
            position,
        })
    }
}

impl PartialEq<Self> for User {
    fn eq(&self, other: &Self) -> bool {
        self.username == other.username
    }
}

impl PartialOrd<Self> for User {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.username.partial_cmp(&other.username)
    }
}

unsafe impl Send for SafePointer<User> {}
unsafe impl Sync for SafePointer<User> {}