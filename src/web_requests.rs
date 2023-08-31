use std::{env, fs};
use std::ops::Deref;
use percent_encoding::percent_decode_str;
use rocket::{get, post, State};
use rocket::response::stream::{Event, EventStream};
use crate::server;
use crate::state::{ConsoleState, ServerState};
use rocket::tokio::time;

#[get("/get_console")] // Go to link for infinite download lul
pub(crate) fn get_console(state: &State<ConsoleState>) -> EventStream![Event + '_] {
    EventStream! {
        let mut interval = time::interval(std::time::Duration::from_millis(100));
        loop {
            let mut result = String::new();
            result.push_str("<h3>");
            for log in state.logs.lock().unwrap().iter() {
                let log = log.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;");
                result.push_str(format!("{}<br>", log).as_str());
            }
            result.push_str("</h3>");
            yield Event::data(result);
            interval.tick().await;
        }
    }
}

#[post("/send_command", data = "<command>")]
pub(crate) fn send_command(state: &State<ConsoleState>, command: String) -> String {
    let command = command[8..].to_string();
    let binding = percent_decode_str(command.as_str()).decode_utf8().unwrap();
    let formatted_command = binding.deref();
    let mut input = state.input.lock().unwrap();
    input.push_str(formatted_command);
    input.push('\n');
    let mut result = String::new();
    result.push_str("<input type=\"text\" name=\"command\" placeholder=\"Command\" autofocus onfocus=\"this.select()\"/>"); // Escape characters go brrrr
    result.push_str("<input type=\"submit\" value=\"Send\" />");
    result
}

#[post("/start_server", data= "<server_location>")]
pub(crate) fn start_server(server_state: &State<ServerState>, console_state: &State<ConsoleState>, mut server_location: String) -> Result<(), String> {
    match server_state.child.lock().unwrap().as_ref() {
        Some(_) => {
            return Err("Server already running".to_string());
        },
        None => {},
    }
    server_location = server_location.strip_prefix("loc=").expect("Invalid data").to_string();
    if server_location.contains("..") {
        return Err("No directory traversal for you".to_string());
    }
    println!("Starting");
    let logs_clone = console_state.logs.clone();
    let input_clone = console_state.input.clone();
    let child_clone = server_state.child.clone();
    std::thread::spawn(move || {
        server::start_server(logs_clone, input_clone, child_clone, server_location.to_string());
    });
    Ok(())
}

#[get("/get_servers")]
pub(crate) fn get_servers() -> String {
    env::set_current_dir("server").expect("No server folder");
    let paths = fs::read_dir("./").unwrap();
    let dirs = paths
        .filter_map(|e| {
            e.ok().and_then(|d| {
                let p = d.path();
                if p.is_dir() {
                    Some(p)
                } else {
                    None
                }
            })
        });
    let mut result = String::new();
    for dir in dirs {
        let dir_path = dir.strip_prefix("./").unwrap().to_str().unwrap();
        let new_option = format!(
            "<option value=\"{}\">{}</option>",
            dir_path,
            dir_path,
        );
        result.push_str(new_option.as_str());
    }
    env::set_current_dir("..").expect("what");
    result
}

#[post("/stop_server")]
pub(crate) fn stop_server(console_state: &State<ConsoleState>) -> Result<(), String> {
    let command = "stop".to_string();
    let mut input = console_state.input.lock().unwrap();
    input.push_str(command.as_str());
    input.push('\n');
    Ok(())
}

#[post("/kill_server")]
pub(crate) fn kill_server(server_state: &State<ServerState>) -> Result<(), String> {
    match server_state.child.lock().unwrap().as_ref() {
        Some(_) => {
            server::kill_server(server_state.child.clone()).map_err(|e| e.to_string())
        },
        None => {
            return Err("Server not running".to_string());
        },
    }
}