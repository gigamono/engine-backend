use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use utilities::setup::SharedSetup;

pub struct BackendServer {
    setup: Arc<Mutex<SharedSetup>>,
}

impl BackendServer {
    pub fn new(setup: Arc<Mutex<SharedSetup>>) -> Self {
        Self { setup }
    }

    pub fn enable_subscriptions(&self) {
        let nc = &self.setup.as_ref().lock().unwrap().nats.conn;

        let sub = nc.queue_subscribe("v1.run.workspaces", "message").unwrap();

        for msg in sub.timeout_iter(Duration::from_secs(100)) {
            println!("Received {}", &msg);
            msg.respond("responded from egine-backend");
        }

        // The closure here needs to become tokio future.
        // let _ = sub.with_handler(|msg| {
        //     msg.respond("responded from egine-backend");
        //     Ok(())
        // });
    }
}
