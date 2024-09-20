mod my_vector;
mod api;
mod models;

use std::net::IpAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use rocket::{get, routes, tokio, uri, Config};
use rocket::fs::NamedFile;
use rocket::http::Status;
use rocket::response::Redirect;
use rocket::yansi::Paint;
use crate::api::{create_datapoint, create_server, create_test, create_user, delete_datapoint, delete_server, delete_test, delete_user, get_datapoint_info, get_server_info, get_servers, get_servers_manager, get_test_data, get_test_info, get_tests, get_user_info, get_users, update_datapoint, update_server, update_test, update_user};
use crate::models::SiteData;
use crate::my_vector::MyVector;

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {

    let mut user_list = MyVector::load_from_file("./data/users").await; // Load users
    
    user_list.quick_sort().await;
    user_list.save_to_file("./data/users").await.expect("Cannot save users!"); // Ensure users are sorted on start
    
    let mut server_list = MyVector::load_from_file("./data/servers").await; // Load servers 
    
    server_list.quick_sort().await;
    server_list.save_to_file("./data/servers").await.expect("Cannot save servers!"); // Ensure servers are sorted on start
    
    let site_data = Arc::new(Mutex::new(SiteData {
        users: user_list,
        servers: server_list,
    }));

    let _ = rocket::build() // Create a new webserver
        .mount("/api", routes![
            get_users, get_servers,
            get_servers_manager, get_tests,
            get_test_data, get_server_info,
            update_server, delete_server,
            create_server, update_user,
            get_user_info, delete_user,
            create_user, get_datapoint_info,
            update_test, create_test, delete_test,
            update_datapoint, create_datapoint, delete_datapoint,
            get_test_info,
        ]) // All API calls
        .mount("/", routes![index, login, catch_all]) // All public-facing pages
        .manage(site_data) // Share the site data with the web-server, so that data can be shown to the user
        .launch() // Start the web server
        .await?;

    Ok(())
}

#[get("/<path..>")] // Ran when accessing anything not specified. Checks the ./public directory
async fn catch_all(path: PathBuf) -> Result<NamedFile, Status> {
    let mut file_path = path.clone();

    if file_path.extension().is_none() { // If file has no extension, assume html file
        file_path.set_extension("html");
    }

    let path = format!("./public/{}", file_path.display());
    let path = PathBuf::from(path);
    NamedFile::open(path) //Gets the path from the ./public file
        .await
        .map_err(|_| Status::NotFound) // Return not found on an error
}

#[get("/login")] // Ran when accessing /login, used for redirecting from index
async fn login() -> NamedFile {
    NamedFile::open("./public/login.html").await.expect("Could not find file!")
}

#[get("/")] // Ran when accessing /, redirects to the login page
async fn index() -> Redirect {
    Redirect::to(uri!(login))
}