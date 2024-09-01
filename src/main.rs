mod my_vector;
mod api;
mod models;

use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use rocket::{get, routes, tokio, uri};
use rocket::fs::NamedFile;
use rocket::http::Status;
use rocket::response::Redirect;
use crate::api::{create_server, delete_server, get_server_info, get_servers, get_servers_manager, get_test_data, get_tests, get_users, update_server};
use crate::models::SiteData;
use crate::my_vector::MyVector;

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {

    let user_list = MyVector::load_from_file("./data/users").await;
    let server_list = MyVector::load_from_file("./data/servers").await;
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
            create_server,
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