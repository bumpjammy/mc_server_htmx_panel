use std::process::Child;
use std::sync::{Arc, Mutex};

pub(crate) struct ConsoleState {
    pub(crate) logs: Arc<Mutex<Vec<String>>>,
    pub(crate) input: Arc<Mutex<String>>,
}

pub(crate) struct ServerState {
    pub(crate) child: Arc<Mutex<Option<Child>>>,
}