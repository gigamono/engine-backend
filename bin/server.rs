extern crate engine_backend;
extern crate utilities;

use std::sync::{Arc, Mutex};

use engine_backend::BackendServer;
use utilities::setup::SharedSetup;

fn main() {
    let setup = Arc::new(Mutex::new(SharedSetup::new().unwrap()));
    let server = BackendServer::new(setup);
    server.enable_subscriptions()
}
