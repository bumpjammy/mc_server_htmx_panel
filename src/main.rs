mod state;
mod web_requests;
mod server;

use std::sync::Arc;
use rocket::{launch, routes};
use rocket::config::LogLevel::Critical;
use std::sync::Mutex;
use rocket::fs::FileServer;

#[launch]
fn rocket() -> _ {
    let console_state = state::ConsoleState {
        logs: Arc::new(Mutex::new(Vec::new())),
        input: Arc::new(Mutex::new(String::new())),
    };

    let server_state = state::ServerState {
        child: Arc::new(Mutex::new(None)),
    };

    let config = rocket::Config::figment()
        .merge(("log_level", Critical));
    rocket::custom(config)
        .manage(console_state)
        .manage(server_state)
        .mount("/", FileServer::from("webpages/"))
        .mount("/api/", routes![
            web_requests::get_console,
            web_requests::send_command,
            web_requests::start_server,
            web_requests::stop_server,
            web_requests::kill_server,
        ])
}