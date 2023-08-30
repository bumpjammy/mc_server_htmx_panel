use std::env;
use std::process::Child;
use std::io::{BufRead, Result, Write};
use std::sync::{Arc, Mutex};

fn create_child_server_process() -> Result<Child> {
    env::set_current_dir("server")?;
    let child = std::process::Command::new("java")
        .arg("-jar")
        .arg("server.jar")
        .arg("--nogui")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn();
    env::set_current_dir("..")?;
    return child;
}

pub(crate) fn start_server(logs: Arc<Mutex<Vec<String>>>, input: Arc<Mutex<String>>, child: Arc<Mutex<Option<Child>>>) {
    logs.lock().unwrap().clear();
    input.lock().unwrap().clear();
    let new_child = create_child_server_process().expect("Failed to start server");
    *child.lock().unwrap() = Some(new_child);

    let child_clone = child.clone();
    std::thread::spawn({
        let logs_clone = Arc::clone(&logs);
        move || {
            let mut stdout = std::io::BufReader::new(child_clone.lock().unwrap().as_mut().unwrap().stdout.take().unwrap());
            let mut buffer = String::new();
            loop {
                buffer.clear();
                match stdout.read_line(&mut buffer) {
                    Ok(_) => {
                        if buffer == "" { // This caused me so much pain
                            break;
                        }
                        let mut logs = logs_clone.lock().unwrap();
                        logs.push(buffer.clone());
                    },
                    Err(_) => {
                        break;
                    },
                }
            }
        }
    });

    let mut buffer = String::new();
    let child_clone = child.clone();
    while buffer.trim() != "stop" {

        match child_clone.lock().unwrap().as_mut().unwrap().stdin.as_mut() {
            Some(stdin) => {
                stdin.write_all(buffer.as_bytes()).expect("Failed to write to child stdin");
                stdin.flush().expect("Failed to flush child stdin");
            },
            None => eprintln!("Failed to get child stdin"),
        }
        buffer = input.lock().unwrap().clone();
        input.lock().unwrap().clear();
    }

    child.lock().unwrap().as_mut().unwrap().stdin.as_mut().unwrap().write_all(b"stop\n").expect("Failed to write to child stdin");
    child.lock().unwrap().as_mut().unwrap().wait().expect("Failed to wait for child");
    *child.lock().unwrap() = None;
    input.lock().unwrap().clear();
}

pub(crate) fn kill_server(child: Arc<Mutex<Option<Child>>>) -> Result<()> {
    let mut locked_child = child.lock().unwrap();
    locked_child.as_mut().unwrap().kill()?;
    locked_child.as_mut().unwrap().wait()?;
    *locked_child = None;
    Ok(())
}