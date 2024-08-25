mod my_vector;
mod api;
mod models;

use std::path::PathBuf;
use rocket::{get, routes, uri};
use rocket::fs::NamedFile;
use rocket::http::Status;
use rocket::response::Redirect;


#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let _ = rocket::build() // Create a new webserver
        .mount("/", routes![index, login, catch_all]) // Redirect a path to the correct function
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

#[get("/")] // Rand when accessing /, redirects to the login page
async fn index() -> Redirect {
    Redirect::to(uri!(login))
}