// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

use super::handlers;
use futures::FutureExt;
use log::{error, info};
use std::{future::Future, panic::AssertUnwindSafe, sync::Arc};
use tokio::task;
use utilities::{
    natsio::{self, Message, WorkspacesAction},
    result::{Context, HandlerResult, Result},
    setup::SharedSetup,
};

pub struct BackendNatsServer {
    setup: Arc<SharedSetup>,
}

impl BackendNatsServer {
    pub fn new(setup: Arc<SharedSetup>) -> Self {
        Self { setup }
    }

    pub async fn listen(&self) -> Result<()> {
        // Initialize logger.
        env_logger::init();

        // Get config.
        let config = &self.setup.config;

        // Get subscribed target.
        let sub_target = natsio::get_backend_first_sub_target(config, WorkspacesAction::RunSurl);

        // Get workspace subject.
        let subject =
            natsio::create_workpaces_subject(&config, WorkspacesAction::RunSurl, sub_target);

        info!(r#"Subscribing to subject "{}""#, subject);

        // Get nats connection object.
        let nats_conn = &self.setup.nats;

        // Queue-subscribe to subject.
        let subscription = nats_conn
            .queue_subscribe(&subject, natsio::DEFAULT_QUEUE_GROUP_NAME)
            .await
            .context(format!(r#"queue subscribing to subject, "{}""#, subject))?; // TODO(appcypher): need get_workpace_subject_responder

        // Create a Sync subscription.
        let sub = Arc::new(subscription);

        // Create a local task set to run tasks on the current thread because V8 Isolate (an some others) used in message handlers are !Send.
        let local = task::LocalSet::new();

        // Handle messages infinitely.
        // TODO(appcypher):
        // Right now everything runs on single thread due to LocalTaskSet. Find a way to create futures on multiple threads and start them on their respective thread.
        // Maybe start a different os thread on each iteration. Maybe Rayon. No clue yet.
        loop {
            // Get next message. Panics if connection closed or subscription canceled.
            let msg = sub
                .next()
                .await
                .context("connection closed or subscription canceled")?;

            info!(
                r#"New message {{ subject: "{}"; reply: {:?} }}"#,
                msg.subject, msg.reply
            );

            // Run the local task on current thread. And catch error before it propagates.
            local
                .run_until(local.spawn_local(Self::error_wrap(
                    handlers::run_surl,
                    Arc::clone(&self.setup),
                    msg,
                )))
                .await
                .context("running local task")?
        }
    }

    async fn error_wrap<F, Fut>(func: F, setup: Arc<SharedSetup>, msg: Message)
    where
        F: FnOnce(Arc<SharedSetup>, Arc<Message>) -> Fut,
        Fut: Future<Output = HandlerResult<Vec<u8>>>,
    {
        let msg = Arc::new(msg);

        // AssertUnwindSafe to catch handler panics and log errors.
        match AssertUnwindSafe(func(setup, Arc::clone(&msg)))
            .catch_unwind()
            .await
        {
            // Handler returned a result.
            Ok(Ok(response)) => {
                // Send response. Ignore error if unable to do that.
                if let Err(err) = msg
                    .respond(response)
                    .await
                    .context("sending message via reply channel")
                {
                    error!("{:?}", err);
                };
            }
            Ok(Err(err)) => {
                // Log error.
                error!("{:?}", err);

                // TODO(appcypher): Generating an http request from HandlerError
                // Send appropriate server response. Ignore error if unable to do that.
                if let Err(err) = msg
                    .respond(b"Placeholder <An error occured>")
                    .await
                    .context("sending error message via reply channel")
                {
                    error!("{:?}", err);
                };
            }
            // Handler panicked.
            Err(err) => {
                // We catch panics, just to log the error.
                error!("{:?}", err);
            }
        }
    }
}
