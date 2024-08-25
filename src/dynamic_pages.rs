use std::sync::{Arc};
use rocket::{get, State};
use rocket::tokio::sync::Mutex;
use crate::models::SiteData;

#[get("/test-list/<server_id>")]
pub async fn test_list_page(site_data: &State<Arc<Mutex<SiteData>>>, server_id: String) {
    let site_data = site_data.lock().await;
    
}