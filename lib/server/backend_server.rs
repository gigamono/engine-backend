use crate::handlers;
use log::{error, info};
use std::sync::Arc;
use tokio::task;
use utilities::{
    messages::error::SystemError,
    natsio::{self, Message, WorkspacesAction},
    result::{HandlerResult, Result},
    setup::SharedSetup,
};

pub struct BackendServer {
    setup: Arc<SharedSetup>,
}

impl BackendServer {
    pub fn new(setup: Arc<SharedSetup>) -> Self {
        Self { setup }
    }

    pub async fn enable_subscriptions(&self) -> Result<()> {
        // Initialize logger.
        env_logger::init();

        // Get config.
        let config = &self.setup.config;

        // Get subscribed target.
        let sub_target = natsio::get_backend_first_sub_target(config, WorkspacesAction::RunSurl);

        // Get workspace subject.
        let subject = natsio::get_workpace_subject(&config, WorkspacesAction::RunSurl, sub_target);

        info!(r#"Subscribing to subject "{}""#, subject);

        // Get nats connection object.
        let nats_conn = &self.setup.nats;

        // Queue-subscribe to subject.
        let subscription =
            nats_conn.queue_subscribe(&subject, "v1.run_surl.workspace_responder")?; // TODO: need get_workpace_subject_responder

        // Create a ref-counted subscription.
        let arc_sub = Arc::new(subscription);

        // Clone subscription for use in a separate thread.
        let arc_sub = Arc::clone(&arc_sub);

        // Clone setup for spawn_block.
        let setup = Arc::clone(&self.setup);

        // Start a blocking thread for `arc_sub.next` calls.
        task::spawn_blocking(move || {
            loop {
                // Panics if connection closed or subscription cancelled.
                let msg = arc_sub.next().unwrap();

                // Clone setup for async block.
                let setup = Arc::clone(&setup);

                // Spawn task.
                tokio::spawn(async move {
                    // Call handler function and check error.
                    let result = handlers::run_surl(setup, &msg).await;
                    Self::check_error(result, &msg);
                });
            }
        })
        .await
        .map_err(|err| SystemError::Join {
            ctx: "unable to start a blocking thread for `next` calls".to_string(),
            src: err,
        })?
    }

    fn check_error(e: HandlerResult<()>, msg: &Message) {
        if let Err(e) = e {
            error!("{}", e);
            msg.respond(b"????").unwrap(); // TODO: Send error message.
        }
    }
}
