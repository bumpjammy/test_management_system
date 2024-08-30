use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::path::Path;
use std::str::FromStr;
use rocket::tokio::fs;
use crate::models::{Server, Test};
use crate::my_vector::{MyVector, SafePointer};

impl Server {
    pub fn new(id: String, name: String, created_by: String, ram: u32, cpu: u32) -> Self {
        Self {
            id,
            name,
            created_by,
            ram,
            cpu,
            tests: MyVector::new(),
        }
    }

    pub fn get_id(&self) -> String {
        self.id.clone()
    }
    
    pub fn get_name(&self) -> String {
        self.name.clone()
    }
    
    pub fn get_created_by(&self) -> String {
        self.created_by.clone()
    }
    
    pub fn get_ram(&self) -> u32 {
        self.ram
    }
    
    pub fn get_cpu(&self) -> u32 {
        self.cpu
    }

    pub async fn set_id(&mut self, new_id: String) {
        let old_path = format!("./data/tests/{}", self.id);
        let new_path = format!("./data/tests/{}", new_id);

        if Path::new(&old_path).exists() {
            if !Path::new(&new_path).exists() {
                fs::create_dir_all(&new_path).await.unwrap();
            }

            let mut entries = fs::read_dir(&old_path).await.unwrap();
            while let Ok(Some(entry)) = entries.next_entry().await {
                let file_path = entry.path();
                let file_name = file_path.file_name().unwrap();
                let new_file_path = Path::new(&new_path).join(file_name);

                // Move the file to the new directory
                fs::rename(file_path, new_file_path).await.unwrap();
            }

            // Remove the old directory
            fs::remove_dir_all(&old_path).await.unwrap();
        }

        self.id = new_id;
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name
    }

    pub fn set_created_by(&mut self, created_by: String) {
        self.created_by = created_by
    }

    pub fn set_ram(&mut self, ram: u32) {
        self.ram = ram
    }

    pub fn set_cpu(&mut self, cpu: u32) {
        self.cpu = cpu
    }
    
    pub async fn load_tests(&mut self) {
        let path = format!("./data/tests/{}", self.id);

        if !Path::new(&path).exists() {
            fs::create_dir_all(&path).await.unwrap();
        }

        let mut tests = MyVector::new();

        let mut entries = fs::read_dir(&path).await.unwrap();

        while let Ok(Some(entry)) = entries.next_entry().await {
            let file_path = entry.path();

            if let Some(test_id) = file_path.file_stem().and_then(|s| s.to_str()) {
                let data = MyVector::load_from_file(file_path.to_str().unwrap()).await;
                let mut test = Test::new(test_id.to_string());
                test.data = data;
                tests.push(test).await;
            }
        }
        
        self.tests = tests;
    }
}

impl Display for Server {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = format!("{},{},{},{},{}",
            self.id,
            self.name,
            self.created_by,
            self.ram,
            self.cpu,
        );

        write!(f, "{}", str)
    }
}

impl FromStr for Server {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(',').collect();
        if parts.len() != 5 {
            return Err(())
        }
        
        Ok(Self {
            id: parts[0].to_string(),
            name: parts[1].to_string(),
            created_by: parts[2].to_string(),
            ram: u32::from_str(parts[3]).map_err(|_| ())?,
            cpu: u32::from_str(parts[4]).map_err(|_| ())?,
            tests: MyVector::new(),
        })
    }
}
impl PartialEq for Server {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl PartialOrd for Server {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.id.partial_cmp(&other.id)
    }
}

unsafe impl Send for SafePointer<Server> {}
unsafe impl Sync for SafePointer<Server> {}