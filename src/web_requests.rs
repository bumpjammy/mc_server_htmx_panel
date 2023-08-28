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

#[post("/start_server")]
pub(crate) fn start_server(server_state: &State<ServerState>, console_state: &State<ConsoleState>) -> Result<(), String> {
    match server_state.child.lock().unwrap().as_ref() {
        Some(_) => {
            return Err("Server already running".to_string());
        },
        None => {},
    }
    println!("Starting");
    let logs_clone = console_state.logs.clone();
    let input_clone = console_state.input.clone();
    let child_clone = server_state.child.clone();
    std::thread::spawn(move || {
        server::start_server(logs_clone, input_clone, child_clone);
    });
    Ok(())
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