use std::fmt::format;
use std::str::FromStr;
use std::sync::Arc;
use rocket::{delete, get, post, FromForm, State};
use rocket::form::Form;
use rocket::http::hyper::Response;
use rocket::http::Status;
use rocket::response::content::RawHtml;
use rocket::tokio::signal::unix::signal;
use rocket::tokio::sync::Mutex;
use rocket::yansi::Paint;
use crate::models::{Server, SiteData, Test};

#[get("/get_users")]
pub async fn get_users(site_data: &State<Arc<Mutex<SiteData>>>) -> RawHtml<String> {
    let site_data = site_data.lock().await;

    let mut output = String::new();
    output.push_str("<table>");
    output.push_str("<tr><th>username</th><th>forename</th><th>surname</th><th>position</th></tr>\n");

    for i in 0..site_data.users.length {
        let user = site_data.users.get(i).await.unwrap();

        output.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>\n",
            user.get_username(), user.get_forename().unwrap_or_default(), user.get_surname().unwrap_or_default(), user.get_position()
        ));
        
    }
    output.push_str("</table>");

    RawHtml(output)
}

#[get("/get_servers")]
pub async fn get_servers(site_data: &State<Arc<Mutex<SiteData>>>) -> RawHtml<String> {
    get_server_table(site_data, "/test-list").await
}

#[get("/get_servers_manager")]
pub async fn get_servers_manager(site_data: &State<Arc<Mutex<SiteData>>>) -> RawHtml<String> {
    get_server_table(site_data, "/manage-server").await
}

async fn get_server_table(site_data: &State<Arc<Mutex<SiteData>>>, url_prefix: &str) -> RawHtml<String> {
    let site_data = site_data.lock().await;
    let servers = &site_data.servers;
    let mut output = String::new();

    // Start the HTML table
    output.push_str("<table>");
    output.push_str("<tr>");
    output.push_str("<th>ID</th>");
    output.push_str("<th>Name</th>");
    output.push_str("<th>Created By</th>");
    output.push_str("<th>RAM (MB)</th>");
    output.push_str("<th>CPU (Cores)</th>");
    output.push_str("<th>Number of Tests</th>");
    output.push_str("</tr>");

    // Iterate through servers and add rows to the table
    for i in 0..servers.length {
        let mut server = servers.get(i).await.unwrap();
        server.load_tests().await;
        let length = server.tests.length;

        // Start the clickable row with an anchor tag
        output.push_str(&format!(
            "<tr onclick=\"window.location.href='{}?server_id={}'\" style=\"cursor:pointer\">",
            url_prefix,
            server.get_id(),
        ));

        // Add server data to the row
        output.push_str(&format!("<td>{}</td>", server.get_id()));
        output.push_str(&format!("<td>{}</td>", server.get_name()));
        output.push_str(&format!("<td>{}</td>", server.get_created_by()));
        output.push_str(&format!("<td>{}</td>", server.get_ram()));
        output.push_str(&format!("<td>{}</td>", server.get_cpu()));
        output.push_str(&format!("<td>{}</td>", length));

        // Close the row
        output.push_str("</tr>");
    }

    // End the HTML table
    output.push_str("</table>");

    RawHtml(output)
}

#[get("/get_server_info/<server_id>")]
pub async fn get_server_info(site_data: &State<Arc<Mutex<SiteData>>>, server_id: String) -> String {
    let site_data = site_data.lock().await;

    let server_index = match site_data.servers.search(|a| a.get_id() == server_id).await {
        Some(server) => server,
        None => return "Server Not Found".to_string(),
    };

    let server = site_data.servers.get(server_index).await.unwrap();
    let mut output = String::new();

    output.push_str(server.get_id().as_str());
    output.push_str(",");
    output.push_str(server.get_name().as_str());
    output.push_str(",");
    output.push_str(server.get_created_by().as_str());
    output.push_str(",");
    output.push_str(server.get_ram().to_string().as_str());
    output.push_str(",");
    output.push_str(server.get_cpu().to_string().as_str());

    output
}

#[get("/get_tests/<server_id>")]
pub async fn get_tests(site_data: &State<Arc<Mutex<SiteData>>>, server_id: String) -> RawHtml<String> {
    let site_data = site_data.lock().await;

    // Search for the server by server_id
    let server_index = match site_data.servers.search(|a| a.get_id() == server_id).await {
        Some(server) => server,
        None => return RawHtml("Server not found!".to_string())
    };

    // Get the server object
    let mut server = site_data.servers.get(server_index).await.unwrap();
    server.load_tests().await;

    // Start the HTML table
    let mut output = String::new();
    output.push_str("<table>");
    output.push_str("<tr>");
    output.push_str("<th>ID</th>");
    output.push_str("<th>Num. of Data Points</th>");
    output.push_str("</tr>");

    // Iterate through the tests and add rows to the table
    for i in 0..server.tests.length {
        let test = server.tests.get(i).await.unwrap();
        let test_id = test.get_id();
        let num_data_points = test.data.length;

        let row_link = format!("/test-data?server_id={}&test_id={}", server_id, test_id);
        output.push_str(&format!(
            "<tr onclick=\"window.location.href='{}'\" style=\"cursor:pointer\">",
            row_link
        ));
        
        output.push_str(&format!("<td>{}</td>", test_id));
        output.push_str(&format!("<td>{}</td>", num_data_points));
        output.push_str("</tr>");
    }

    // End the HTML table
    output.push_str("</table>");

    RawHtml(output)
}

#[get("/get_test_data/<server_id>/<test_id>")]
pub async fn get_test_data(
    site_data: &State<Arc<Mutex<SiteData>>>,
    server_id: String,
    test_id: String
) -> RawHtml<String> {
    let test = match get_test(site_data.inner(), server_id, test_id).await {
        Some(test) => test,
        None => return RawHtml("Could not find test!".to_string())
    };

    let mut output = String::new();

    // Start the HTML table with a border and add headers
    output.push_str("<table>");
    output.push_str("<tr>");
    output.push_str("<th>Time</th>");
    output.push_str("<th>RAM (MB)</th>");
    output.push_str("<th>CPU (%)</th>");
    output.push_str("<th>Comment</th>");
    output.push_str("</tr>");

    // Iterate through the test data points and add rows to the table
    for i in 0..test.data.length {
        if let Some(data_point) = test.data.get(i).await {
            output.push_str("<tr>");

            output.push_str(&format!("<td>{}</td>", data_point.get_time()));
            output.push_str(&format!("<td>{}</td>", data_point.get_ram()));
            output.push_str(&format!("<td>{}</td>", data_point.get_cpu()));
            output.push_str(&format!("<td>{}</td>", data_point.get_comment().unwrap_or_default()));

            output.push_str("</tr>");
        }
    }

    // Close the HTML table
    output.push_str("</table>");

    RawHtml(output)
}


pub async fn get_test(site_data: &Arc<Mutex<SiteData>>, server_id: String, test_id: String) -> Option<Test> {
    let site_data = site_data.lock().await;
    let servers = &site_data.servers;

    let server_index = match servers.search(|a| a.get_id() == server_id).await {
        Some(server) => server,
        None => return None
    };

    let mut server = servers.get(server_index).await.expect("Server was found but was not in array!");
    server.load_tests().await;

    let tests = &server.tests;

    let test_index = match tests.search(|a| a.get_id() == test_id).await {
        Some(test) => test,
        None => return None
    };

    Some(tests.get(test_index).await.expect("Test was found but was not in array!"))
}

#[derive(FromForm)]
struct UpdateServerData {
    old_id: String,
    id: String,
    name: String,
    created_by: String,
    ram: String,
    cpu: String,
}

#[derive(FromForm)]
struct CreateServerData {
    id: String,
    name: String,
    created_by: String,
    ram: String,
    cpu: String,
}

#[post("/update_server", data = "<form_data>")]
pub async fn update_server(
    site_data: &State<Arc<Mutex<SiteData>>>,
    form_data: Form<UpdateServerData>
) -> Status {
    let site_data = site_data.lock().await;

    let server_index = match site_data.servers.search(|a| a.get_id() == form_data.old_id).await {
        Some(server_index) => server_index,
        None => return Status::NotFound
    };
    let server = site_data.servers.get_mut(server_index).await.unwrap();

    println!("{}", form_data.ram);
    let ram = match u32::from_str(form_data.ram.as_str()) {
        Ok(ram) => ram,
        Err(_) => return Status::UnprocessableEntity,
    };
    println!("{}", form_data.cpu);
    let cpu = match u32::from_str(form_data.cpu.as_str()) {
        Ok(cpu) => cpu,
        Err(_) => return Status::UnprocessableEntity,
    };

    server.set_id(form_data.id.clone()).await;
    server.set_name(form_data.name.clone());
    server.set_created_by(form_data.created_by.clone());
    server.set_ram(ram);
    server.set_cpu(cpu);

    println!("Saving");
    site_data.servers.save_to_file("./data/servers").await.expect("Failed to save servers!");
    for i in 0..site_data.servers.length {
        let server = site_data.servers.get(i).await.unwrap();
        println!("{}", server.get_id());
    }
    println!("Saved!");

    Status::Ok
}

#[delete("/delete_server?<server_id>")]
pub async fn delete_server(site_data: &State<Arc<Mutex<SiteData>>>, server_id: String) -> Status {
    let mut site_data = site_data.lock().await;

    let server_index = match site_data.servers.search(|a| a.get_id() == server_id).await {
        Some(server_index) => server_index,
        None => return Status::NotFound,
    };
    
    let server = site_data.servers.get(server_index).await.unwrap();
    server.delete_tests_directory().await;

    site_data.servers.remove(server_index).await;
    site_data.servers.save_to_file("./data/servers").await.expect("Failed to save servers!");
    Status::Ok
}

#[post("/create_server", data = "<form_data>")]
pub async fn create_server(site_data: &State<Arc<Mutex<SiteData>>>, form_data: Form<CreateServerData>) -> Status {
    let mut site_data = site_data.lock().await;
    
    let ram = match u32::from_str(form_data.ram.as_str()) {
        Ok(ram) => ram,
        Err(_) => return Status::UnprocessableEntity,
    };
    let cpu = match u32::from_str(form_data.cpu.as_str()) {
        Ok(cpu) => cpu,
        Err(_) => return Status::UnprocessableEntity,
    };
    
    let server = Server::new(
        form_data.id.clone(),
        form_data.name.clone(),
        form_data.created_by.clone(), 
        ram.clone(),
        cpu.clone(),
    );
    
    site_data.servers.push(server).await;
    site_data.servers.save_to_file("./data/servers").await.expect("Failed to save servers!");
    
    Status::Ok
}