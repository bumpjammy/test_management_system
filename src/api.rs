use std::char::from_digit;
use std::str::FromStr;
use std::sync::Arc;
use rocket::{delete, get, post, FromForm, State};
use rocket::form::Form;
use rocket::http::Status;
use rocket::response::content::RawHtml;
use rocket::tokio::fs::{create_dir_all, rename};
use rocket::tokio::sync::Mutex;
use crate::models::{DataPoint, Position, ScheduleEntry, Server, SiteData, Test, User};

#[get("/get_users?<search>")]
pub async fn get_users(
    site_data: &State<Arc<Mutex<SiteData>>>,
    search: Option<String>,
) -> RawHtml<String> {
    let site_data = site_data.lock().await;
    let mut output = String::new();
    output.push_str("<table>");
    output.push_str("<tr><th>Username</th><th>Forename</th><th>Surname</th><th>Position</th></tr>\n");

    let users = if let Some(ref search_term) = search {
        site_data
            .users
            .search_all(|user| user.get_username().contains(search_term))
            .await
    } else {
        site_data.users.clone()
    };

    for i in 0..users.length {
        let user = users.get(i).await.unwrap();
        let username = user.get_username();

        output.push_str(&format!(
            "<tr onclick=\"window.location.href='/manage-user?username={}'\" style=\"cursor:pointer\">",
            username
        ));

        output.push_str(&format!(
            "<td>{}</td><td>{}</td><td>{}</td><td>{}</td>",
            username,
            user.get_forename().unwrap_or_default(),
            user.get_surname().unwrap_or_default(),
            user.get_position()
        ));

        output.push_str("</tr>\n");
    }
    output.push_str("</table>");

    RawHtml(output)
}

#[get("/get_servers?<search>")]
pub async fn get_servers(
    site_data: &State<Arc<Mutex<SiteData>>>,
    search: Option<String>,
) -> RawHtml<String> {
    get_server_table(site_data, "/test-list", search).await
}

#[get("/get_servers_manager?<search>")]
pub async fn get_servers_manager(
    site_data: &State<Arc<Mutex<SiteData>>>,
    search: Option<String>,
) -> RawHtml<String> {
    get_server_table(site_data, "/manage-server", search).await
}

async fn get_server_table(
    site_data: &State<Arc<Mutex<SiteData>>>,
    url_prefix: &str,
    search: Option<String>,
) -> RawHtml<String> {
    let site_data = site_data.lock().await;
    let mut output = String::new();

    output.push_str("<table>");
    output.push_str("<tr>");
    output.push_str("<th>ID</th>");
    output.push_str("<th>Name</th>");
    output.push_str("<th>Created By</th>");
    output.push_str("<th>RAM (MB)</th>");
    output.push_str("<th>CPU (Cores)</th>");
    output.push_str("<th>Number of Tests</th>");
    output.push_str("</tr>");

    let servers = if let Some(ref search_term) = search {
        site_data
            .servers
            .search_all(|server| server.get_id().contains(search_term))
            .await
    } else {
        site_data.servers.clone()
    };

    for i in 0..servers.length {
        let mut server = servers.get(i).await.unwrap();
        server.load_tests().await;
        let length = server.tests.length;

        output.push_str(&format!(
            "<tr onclick=\"window.location.href='{}?server_id={}'\" style=\"cursor:pointer\">",
            url_prefix,
            server.get_id(),
        ));

        output.push_str(&format!("<td>{}</td>", server.get_id()));
        output.push_str(&format!("<td>{}</td>", server.get_name()));
        output.push_str(&format!("<td>{}</td>", server.get_created_by()));
        output.push_str(&format!("<td>{}</td>", server.get_ram()));
        output.push_str(&format!("<td>{}</td>", server.get_cpu()));
        output.push_str(&format!("<td>{}</td>", length));

        output.push_str("</tr>");
    }

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

#[get("/get_user_info/<username>")]
pub async fn get_user_info(site_data: &State<Arc<Mutex<SiteData>>>, username: String) -> String {
    let site_data = site_data.lock().await;

    let user_index = match site_data.users.search(|a| a.get_username() == username).await {
        Some(username_index) => username_index,
        None => return "User Not Found".to_string(),
    };

    let user = site_data.users.get(user_index).await.unwrap();
    let mut output = String::new();

    output.push_str(user.get_username().as_str());
    output.push_str(",");
    output.push_str(user.get_forename().unwrap_or_default().as_str());
    output.push_str(",");
    output.push_str(user.get_surname().unwrap_or_default().as_str());
    output.push_str(",");
    output.push_str(user.get_position().to_string().as_str());

    output
}

#[get("/get_tests/<server_id>?<search>")]
pub async fn get_tests(
    site_data: &State<Arc<Mutex<SiteData>>>,
    server_id: String,
    search: Option<String>,
) -> RawHtml<String> {
    let site_data = site_data.lock().await;

    let server_index = match site_data.servers.search(|a| a.get_id() == server_id).await {
        Some(server) => server,
        None => return RawHtml("Server not found!".to_string()),
    };

    let mut server = site_data.servers.get(server_index).await.unwrap();
    server.load_tests().await;

    let mut output = String::new();
    output.push_str("<table>");
    output.push_str("<tr>");
    output.push_str("<th>ID</th>");
    output.push_str("<th>Num. of Data Points</th>");
    output.push_str("</tr>");

    let tests = if let Some(ref search_term) = search {
        server
            .tests
            .search_all(|test| test.get_id().contains(search_term))
            .await
    } else {
        server.tests.clone()
    };

    for i in 0..tests.length {
        let test = tests.get(i).await.unwrap();
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

    output.push_str("</table>");

    RawHtml(output)
}

#[get("/get_test_data/<server_id>/<test_id>?<search>")]
pub async fn get_test_data(
    site_data: &State<Arc<Mutex<SiteData>>>,
    server_id: String,
    test_id: String,
    search: Option<String>,
) -> RawHtml<String> {
    let test = match get_test(site_data.inner(), server_id.clone(), test_id.clone()).await {
        Some(test) => test,
        None => return RawHtml("Could not find test!".to_string()),
    };

    let mut output = String::new();

    output.push_str("<table>");
    output.push_str("<tr>");
    output.push_str("<th>Time</th>");
    output.push_str("<th>RAM (MB)</th>");
    output.push_str("<th>CPU (%)</th>");
    output.push_str("<th>Comment</th>");
    output.push_str("</tr>");

    for i in 0..test.data.length {
        if let Some(data_point) = test.data.get(i).await {
            let time = data_point.get_time();

            if let Some(ref search_term) = search {
                if !time.contains(search_term) {
                    continue;
                }
            }

            let url = format!(
                "/manage-datapoint?server_id={}&test_id={}&time={}",
                server_id, test_id, time
            );

            output.push_str(&format!(
                "<tr onclick=\"window.location.href='{}'\" style=\"cursor:pointer\">",
                url
            ));

            output.push_str(&format!("<td>{}</td>", time));
            output.push_str(&format!("<td>{}</td>", data_point.get_ram()));
            output.push_str(&format!("<td>{}</td>", data_point.get_cpu()));
            output.push_str(&format!(
                "<td>{}</td>",
                data_point.get_comment().unwrap_or_default()
            ));

            output.push_str("</tr>");
        }
    }

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

#[get("/get_datapoint_info/<server_id>/<test_id>/<time>")]
pub async fn get_datapoint_info(site_data: &State<Arc<Mutex<SiteData>>>, server_id: String, test_id: String, time: String) -> String {
    let test = match get_test(site_data, server_id, test_id).await {
        Some(test) => test,
        None => return "Unable to find test".to_string(),
    };

    let datapoint_index = match test.data.search(|a| a.get_time() == time).await {
        Some(datapoint_index) => datapoint_index,
        None => return "Unable to find datapoint".to_string(),
    };
    let datapoint = test.data.get(datapoint_index).await.unwrap();

    let mut output = String::new();

    output.push_str(datapoint.get_time().as_str());
    output.push_str(",");
    output.push_str(datapoint.get_ram().to_string().as_str());
    output.push_str(",");
    output.push_str(datapoint.get_cpu().to_string().as_str());
    output.push_str(",");
    output.push_str(datapoint.get_comment().unwrap_or_default().as_str());

    output
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

#[derive(FromForm)]
struct UpdateTestData {
    server_id: String,
    old_id: String,
    id: String,
}

#[derive(FromForm)]
struct CreateTestData {
    server_id: String,
    id: String,
}

#[post("/update_test", data = "<form_data>")]
pub async fn update_test(
    site_data: &State<Arc<Mutex<SiteData>>>,
    form_data: Form<UpdateTestData>,
) -> Status {
    let site_data = site_data.lock().await;

    let server_index = match site_data.servers.search(|a| a.get_id() == form_data.server_id).await {
        Some(server_index) => server_index,
        None => return Status::NotFound,
    };
    let server = site_data.servers.get_mut(server_index).await.unwrap();
    server.load_tests().await;

    let test_index = match server.tests.search(|a| a.get_id() == form_data.old_id).await {
        Some(test_index) => test_index,
        None => return Status::NotFound,
    };
    let test = server.tests.get_mut(test_index).await.unwrap();

    test.set_id(form_data.id.clone());

    Status::Ok
}

#[post("/create_test", data = "<form_data>")]
pub async fn create_test(
    site_data: &State<Arc<Mutex<SiteData>>>,
    form_data: Form<CreateTestData>,
) -> Status {
    let site_data = site_data.lock().await;

    let server_index = match site_data.servers.search(|a| a.get_id()  == form_data.server_id).await {
        Some(server_index) => server_index,
        None => return Status::NotFound,
    };
    let server = site_data.servers.get_mut(server_index).await.unwrap();

    let test = Test::new(form_data.id.clone());
    println!("{}", format!("./tests/{}/{}", server.get_id(), test.get_id()).as_str());
    test.data.save_to_file(format!("./data/tests/{}/{}", server.get_id(), test.get_id()).as_str()).await.unwrap();
    server.load_tests().await;
    println!("{}", test);

    Status::Ok
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

    let ram = match u32::from_str(form_data.ram.as_str()) {
        Ok(ram) => ram,
        Err(_) => return Status::UnprocessableEntity,
    };
    let cpu = match u32::from_str(form_data.cpu.as_str()) {
        Ok(cpu) => cpu,
        Err(_) => return Status::UnprocessableEntity,
    };

    server.set_id(form_data.id.clone()).await;
    server.set_name(form_data.name.clone());
    server.set_created_by(form_data.created_by.clone());
    server.set_ram(ram);
    server.set_cpu(cpu);

    site_data.servers.save_to_file("./data/servers").await.expect("Failed to save servers!");
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

#[delete("/delete_user?<username>")]
pub async fn delete_user(site_data: &State<Arc<Mutex<SiteData>>>, username: String) -> Status {
    let mut site_data = site_data.lock().await;

    let user_index = match site_data.users.search(|a| a.get_username() == username).await {
        Some(user_index) => user_index,
        None => return Status::NotFound,
    };

    let user = site_data.users.get(user_index).await.unwrap();
    site_data.users.remove(user_index).await;
    site_data.users.save_to_file("./data/users").await.expect("Failed to save users!");
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

#[post("/create_user", data = "<form_data>")]
pub async fn create_user(site_data: &State<Arc<Mutex<SiteData>>>, form_data: Form<CreateUserData>) -> Status {
    let mut site_data = site_data.lock().await;

    let forename = match form_data.forename.is_empty() {
        false => Some(form_data.forename.clone()),
        true => None,
    };

    let surname = match form_data.surname.is_empty() {
        false => Some(form_data.surname.clone()),
        true => None,
    };

    let position = Position::from_str(form_data.position.as_str()).unwrap();

    let user = User::new(
        form_data.username.clone(),
        forename,
        surname,
        position,
    );

    site_data.users.push(user).await;

    Status::Ok
}

#[derive(FromForm)]
struct UpdateUserData {
    old_username: String,
    username: String,
    forename: String,
    surname: String,
    position: String,
}

#[derive(FromForm)]
struct CreateUserData {
    username: String,
    forename: String,
    surname: String,
    position: String,
}


#[post("/update_user", data = "<form_data>")]
pub async fn update_user(
    site_data: &State<Arc<Mutex<SiteData>>>,
    form_data: Form<UpdateUserData>
) -> Status {
    let site_data = site_data.lock().await;

    let user_index = match site_data.users.search(|user| user.get_username() == form_data.old_username).await {
        Some(user_index) => user_index,
        None => return Status::NotFound,
    };
    let user = site_data.users.get_mut(user_index).await.unwrap();

    user.set_username(form_data.username.clone());

    user.set_forename(match form_data.forename.is_empty() {
        true => None,
        false => Some(form_data.forename.clone())
    });

    user.set_surname(match form_data.surname.is_empty() {
        true => None,
        false => Some(form_data.surname.clone())
    });

    user.set_position(Position::from_str(form_data.position.as_str()).unwrap());

    // Save updated user data to a file
    site_data.users.save_to_file("./data/users").await.expect("Failed to save users!");
    Status::Ok
}

// For updating and creating DataPoints
#[derive(FromForm)]
struct UpdateDataPointData {
    server_id: String,
    test_id: String,
    old_time: String,
    time: String,
    cpu: String,
    ram: String,
    comment: String,
}

#[derive(FromForm)]
struct CreateDataPointData {
    server_id: String,
    test_id: String,
    time: String,
    ram: String,
    cpu: String,
    comment: String,
}

// Get Test Info
#[get("/get_test_info/<server_id>/<test_id>")]
pub async fn get_test_info(
    site_data: &State<Arc<Mutex<SiteData>>>,
    server_id: String,
    test_id: String
) -> String {
    let test = match get_test(site_data.inner(), server_id, test_id).await {
        Some(test) => test,
        None => return "Test Not Found".to_string(),
    };

    let mut output = String::new();
    output.push_str(test.get_id().as_str());
    output
}
// Delete Test
#[delete("/delete_test?<server_id>&<test_id>")]
pub async fn delete_test(
    site_data: &State<Arc<Mutex<SiteData>>>,
    server_id: String,
    test_id: String
) -> Status {
    let site_data = site_data.lock().await;

    // Find the server
    let server_index = match site_data.servers.search(|a| a.get_id() == server_id).await {
        Some(index) => index,
        None => return Status::NotFound,
    };
    let mut server = site_data.servers.get_mut(server_index).await.unwrap();
    server.load_tests().await;

    // Find and remove the test
    let test_index = match server.tests.search(|a| a.get_id() == test_id).await {
        Some(index) => index,
        None => return Status::NotFound,
    };
    server.tests.remove(test_index).await;

    // Save the test data to a file
    server.tests.save_to_file(&format!("./data/servers/{}/tests", server.get_id())).await.expect("Failed to save tests!");
    Status::Ok
}

// Update DataPoint
#[post("/update_datapoint", data = "<form_data>")]
pub async fn update_datapoint(
    site_data: &State<Arc<Mutex<SiteData>>>,
    form_data: Form<UpdateDataPointData>
) -> Status {
    let test = match get_test(site_data.inner(), form_data.server_id.clone(), form_data.test_id.clone()).await {
        Some(test) => test,
        None => return Status::NotFound,
    };

    let datapoint_index = match test.data.search(|a| a.get_time() == form_data.old_time).await {
        Some(index) => index,
        None => return Status::NotFound,
    };
    let datapoint = test.data.get_mut(datapoint_index).await.unwrap();

    // Update the data point
    datapoint.set_time(form_data.time.clone());
    datapoint.set_cpu(form_data.cpu.parse::<u32>().unwrap_or(0));
    datapoint.set_ram(form_data.ram.parse::<u32>().unwrap_or(0));
    datapoint.set_comment(if form_data.comment.is_empty() { None } else { Some(form_data.comment.clone()) });

    // Save the data point data to a file
    test.data.save_to_file(&format!("./data/tests/{}/{}", form_data.server_id, form_data.test_id)).await.expect("Failed to save data points!");
    Status::Ok
}

// Create DataPoint
#[post("/create_datapoint", data = "<form_data>")]
pub async fn create_datapoint(
    site_data: &State<Arc<Mutex<SiteData>>>,
    form_data: Form<CreateDataPointData>
) -> Status {
    let mut test = match get_test(site_data.inner(), form_data.server_id.clone(), form_data.test_id.clone()).await {
        Some(test) => test,
        None => return Status::NotFound,
    };

    // Create a new data point
    let datapoint = DataPoint::new(
        form_data.time.clone(),
        form_data.ram.parse::<u32>().unwrap_or(0),
        form_data.cpu.parse::<u32>().unwrap_or(0),
    );

    test.data.push(datapoint).await;

    // Save the data point data to a file
    test.data.save_to_file(&format!("./data/tests/{}/{}", form_data.server_id, form_data.test_id)).await.expect("Failed to save data points!");
    Status::Ok
}

// Delete DataPoint
#[delete("/delete_datapoint?<server_id>&<test_id>&<time>")]
pub async fn delete_datapoint(
    site_data: &State<Arc<Mutex<SiteData>>>,
    server_id: String,
    test_id: String,
    time: String,
) -> Status {
    let mut test = match get_test(site_data.inner(), server_id.clone(), test_id.clone()).await {
        Some(test) => test,
        None => return Status::NotFound,
    };

    let datapoint_index = match test.data.search(|a| a.get_time() == time).await {
        Some(index) => index,
        None => return Status::NotFound,
    };

    // Remove the data point
    test.data.remove(datapoint_index).await;

    // Save the data point data to a file
    test.data.save_to_file(&format!("./data/tests/{}/{}", server_id, test_id)).await.expect("Failed to save data points!");
    Status::Ok
}

/// Struct for creating a new ScheduleEntry
#[derive(FromForm)]
struct CreateScheduleEntryData {
    id: String,
    datetime: String,
    assignees: String,
    test: String,
}

/// Struct for updating an existing ScheduleEntry
#[derive(FromForm)]
struct UpdateScheduleEntryData {
    old_id: String,
    id: String,
    datetime: String,
    assignees: String,
    test: String,
}

/// Get comma-separated schedule entry information
#[get("/get_schedule_entry_info/<schedule_entry_id>")]
pub async fn get_schedule_entry_info(
    site_data: &State<Arc<Mutex<SiteData>>>,
    schedule_entry_id: String
) -> String {
    let site_data = site_data.lock().await;
    let schedule_index = match site_data.schedules.search(|s| s.get_id() == schedule_entry_id).await {
        Some(index) => index,
        None => return "Schedule Entry Not Found".to_string(),
    };
    let schedule = site_data.schedules.get(schedule_index).await.unwrap();
    format!("{},{},{},{}",
            schedule.get_id(),
            schedule.get_datetime(),
            schedule.get_assignees(),
            schedule.get_test()
    )
}

#[get("/get_schedule_entries?<search>")]
pub async fn get_schedule_entries(
    site_data: &State<Arc<Mutex<SiteData>>>,
    search: Option<String>,
) -> RawHtml<String> {
    let site_data = site_data.lock().await;
    let mut output = String::new();

    output.push_str("<table>");
    output.push_str("<tr><th>ID</th><th>DateTime</th><th>Assignees</th><th>Test</th></tr>\n");

    let schedules = if let Some(ref search_term) = search {
        site_data
            .schedules
            .search_all(|s| s.get_id().contains(search_term))
            .await
    } else {
        site_data.schedules.clone()
    };

    for i in 0..schedules.length {
        let schedule = schedules.get(i).await.unwrap();
        let id = schedule.get_id();

        output.push_str(&format!(
            "<tr onclick=\"window.location.href='/manage-scheduleentry?id={}'\" style=\"cursor:pointer\">",
            id
        ));
        output.push_str(&format!(
            "<td>{}</td><td>{}</td><td>{}</td><td>{}</td>",
            id,
            schedule.get_datetime(),
            schedule.get_assignees(),
            schedule.get_test()
        ));
        output.push_str("</tr>\n");
    }

    output.push_str("</table>");
    RawHtml(output)
}

/// Create a new schedule entry
#[post("/create_schedule_entry", data = "<form_data>")]
pub async fn create_schedule_entry(
    site_data: &State<Arc<Mutex<SiteData>>>,
    form_data: Form<CreateScheduleEntryData>
) -> Status {
    let mut site_data = site_data.lock().await;

    if site_data.schedules.search(|s| s.get_id() == form_data.id).await.is_some() {
        return Status::Conflict;
    }

    let schedule = ScheduleEntry::new(
        form_data.id.clone(),
        form_data.datetime.clone().replace('T', " "),
        form_data.assignees.clone(),
        form_data.test.clone(),
    );

    site_data.schedules.push(schedule).await;
    site_data.schedules.save_to_file("./data/schedules").await.expect("Failed to save schedules!");
    Status::Ok
}

/// Update an existing schedule entry
#[post("/update_schedule_entry", data = "<form_data>")]
pub async fn update_schedule_entry(
    site_data: &State<Arc<Mutex<SiteData>>>,
    form_data: Form<UpdateScheduleEntryData>
) -> Status {
    let mut site_data = site_data.lock().await;

    let schedule_index = match site_data.schedules.search(|s| s.get_id() == form_data.old_id).await {
        Some(index) => index,
        None => return Status::NotFound,
    };

    if form_data.old_id != form_data.id {
        if site_data.schedules.search(|s| s.get_id() == form_data.id).await.is_some() {
            return Status::Conflict;
        }
    }

    let schedule = site_data.schedules.get_mut(schedule_index).await.unwrap();
    schedule.set_id(form_data.id.clone());
    schedule.set_datetime(form_data.datetime.clone().replace('T', " "));
    schedule.set_assignees(form_data.assignees.clone());
    schedule.set_test(form_data.test.clone());

    site_data.schedules.save_to_file("./data/schedules").await.expect("Failed to save schedules!");
    Status::Ok
}

/// Delete a schedule entry
#[delete("/delete_schedule_entry?<schedule_entry_id>")]
pub async fn delete_schedule_entry(
    site_data: &State<Arc<Mutex<SiteData>>>,
    schedule_entry_id: String
) -> Status {
    let mut site_data = site_data.lock().await;

    let schedule_index = match site_data.schedules.search(|s| s.get_id() == schedule_entry_id).await {
        Some(index) => index,
        None => return Status::NotFound,
    };

    site_data.schedules.remove(schedule_index).await;
    site_data.schedules.save_to_file("./data/schedules").await.expect("Failed to save schedules!");
    Status::Ok
}