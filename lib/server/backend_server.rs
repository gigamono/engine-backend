use crate::handlers;
use log::info;
use std::sync::Arc;
use tokio::task;
use utilities::{
    messages::error::SystemError,
    nats::{self, WorkspacesAction},
    result::Result,
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

        // Get workspace subject.
        let subj = nats::get_workpace_subject(
            config,
            WorkspacesAction::RunSurl,
            Some("*"), // TODO: Use config.
        );

        info!(r#"Subscribing to subject "{}""#, subj);

        // Get nats connection object.
        let conn = &self.setup.nats.conn;

        // Queue-subscribe to subject.
        let sub = conn
            .queue_subscribe(&subj, "v1.run_surl.workspace_responder") // TODO: need get_workpace_subject_responder
            .map_err(|err| SystemError::Io {
                ctx: "unable to subscribe".to_string(),
                src: err,
            })?;

        // Create a ref-counted subscription.
        let arc_sub = Arc::new(sub);

        // Clone subscription for use in a separate thread.
        let arc_sub = Arc::clone(&arc_sub);

        // Start a blocking thread for `arc_sub.next` calls.
        task::spawn_blocking(move || {
            loop {
                // Panics if connection closed or subscription cancelled.
                let msg = arc_sub.next().unwrap();

                // Push handler to a separate task.
                tokio::spawn(async move { handlers::run_surl(msg) });
            }
        })
        .await
        .map_err(|err| SystemError::Join {
            ctx: "unable to start a blocking thread for `next` calls".to_string(),
            src: err,
        })?
    }
}
