use crate::handlers;
use log::{error, info};
use std::sync::Arc;
use tokio::{runtime::Runtime, task};
use utilities::{
    natsio::{self, Message, WorkspacesAction},
    result::{Context, HandlerResult, Result},
    setup::SharedSetup,
};

pub struct BackendServer {
    setup: Arc<SharedSetup>,
}

impl BackendServer {
    pub fn new(setup: Arc<SharedSetup>) -> Self {
        Self { setup }
    }

    pub async fn handle_subscriptions(&self) -> Result<()> {
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
            nats_conn.queue_subscribe(&subject, "v1.run_surl.workspace_responder")?; // TODO(appcypher): need get_workpace_subject_responder

        // Create a ref-counted subscription.
        let arc_sub = Arc::new(subscription);

        // Handle messages infinitely.
        loop {
            // Clone setup for spawn_block.
            let setup = Arc::clone(&self.setup);

            // Clone subscription for use in a separate thread.
            let arc_sub = Arc::clone(&arc_sub);

            // Create a reusable tokio runtime.
            let mut rt =
                Runtime::new().expect("creating a new tokio runtime for handling requests");

            // Start a blocking thread for each `arc_sub.next` call.
            task::spawn_blocking(move || {
                // Panics if connection closed or subscription canceled.
                let msg = arc_sub
                    .next()
                    .expect("connection closed or subsscription canceled");

                info!(
                    r#"New message {{ subject: "{}"; reply: {:?} }}"#,
                    msg.subject, msg.reply
                );

                // Spawn task as V8 Isolate is !Send.
                let local = task::LocalSet::new();

                // Run the local task set.
                local.block_on(&mut rt, async move {
                    task::spawn_local(async move {
                        // Call handler function and check error.
                        let result = handlers::run_surl(setup, &msg).await;
                        Self::check_error(result, &msg);
                    })
                    .await
                    .expect("spawning request handling task on local thread")
                });
            })
            .await
            .context("starting a blocking thread for `next` calls")?
        }
    }

    fn check_error(e: HandlerResult<()>, msg: &Message) {
        if let Err(e) = e {
            error!("{:?}", e);
            msg.respond(b"????").expect("replying client"); // TODO(appcypher): Send error message.
        }
    }
}
