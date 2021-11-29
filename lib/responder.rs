// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

use std::pin::Pin;

use futures::{Future, FutureExt};
use tera::events::HttpResponder;
use utilities::{
    errors,
    http::Response,
    natsio::Message,
    result::{Context, Result},
};

pub struct NatsResponder {
    message: Message,
    responded_already: bool,
}

impl NatsResponder {
    pub fn new(message: Message) -> Self {
        Self {
            message,
            responded_already: false,
        }
    }
}

impl HttpResponder for NatsResponder {
    fn respond(self, response: Response) -> Pin<Box<dyn Future<Output = Result<()>>>> {
        async move {
            if self.responded_already {
                errors::any_error("already responded to event")?;
            };

            let response_bytes =
                bincode::serialize(&response).context("serializing http response for nats")?;
            self.message.respond(&response_bytes);
            self.responded_already = true;

            Ok(())
        }
        .boxed_local()
    }
}
