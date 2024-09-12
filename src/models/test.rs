use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::path::Path;
use std::str::FromStr;
use rocket::tokio::runtime::Runtime;
use crate::models::{DataPoint, Test};
use crate::my_vector::{MyVector, SafePointer};

impl Test {
    pub fn new(id: String) -> Self {
        Self {
            id,
            data: MyVector::new(),
        }
    }
    
    pub fn get_id(&self) -> String {
        self.id.clone()
    }

    pub fn set_id(&mut self, id: String) {
        self.id = id;
    }
}

impl Display for Test {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl FromStr for Test {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Use the input string `s` as the `id`
        let id = s.to_string();

        // Initialize `data` as an empty MyVector
        let mut data = MyVector::new();

        // Construct the file path
        let path = format!("./data/tests/{}", id);
        let path_obj = Path::new(&path);

        // Check if the file exists, if not, leave vector empty
        if path_obj.exists() {
            // Open the file
            let file = File::open(&path).map_err(|_| ())?;
            let reader = io::BufReader::new(file);

            // Iterate over each line in the file
            for line_result in reader.lines() {
                let line = line_result.map_err(|_| ())?; // Handle I/O errors
                if !line.trim().is_empty() {
                    // Parse the line into a DataPoint
                    let data_point = line.parse::<DataPoint>().map_err(|_| ())?;
                    // Add the DataPoint to `data`
                    let rt = Runtime::new().unwrap();
                    let _ = rt.block_on(data.push(data_point));
                }
            }
        }

        Ok(Test { id, data }) // Return the constructed Test instance
    }
}

impl PartialEq for Test {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id // Check if ids are equal, if so, tests are equal
    }
}

impl PartialOrd for Test {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.id.partial_cmp(&other.id) // Compare ids to sort
    }
}

unsafe impl Send for SafePointer<Test> {}
unsafe impl Sync for SafePointer<Test> {}