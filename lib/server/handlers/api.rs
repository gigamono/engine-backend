// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

use crate::runtimes::ApiRuntime;
use std::{rc::Rc, sync::Arc};
use tokio::sync::mpsc::Sender;
use utilities::{
    errors::{self, HandlerError, HandlerErrorMessage},
    http,
    hyper::{Body, Request, Response, StatusCode},
    result::HandlerResult,
    setup::CommonSetup,
};

/// The /api/ route handler.
pub struct ApiHandler;

impl ApiHandler {
    /// Starts the runtime that eventually executes the user-defined /api/ handler.
    pub async fn handle(
        request: Request<Body>,
        response_tx: Rc<Sender<Response<Body>>>,
        setup: Arc<CommonSetup>,
    ) -> HandlerResult<()> {
        // Create api runtime.
        let mut api_rt = ApiRuntime::new(request, response_tx, setup)
            .await
            .map_err(http::internal_error)?;

        // Execute api runtime.
        if !api_rt.execute().await.map_err(http::internal_error)? {
            // If result is false, then one of auth or middleware failed.
            return Err(HandlerError::Client {
                ctx: HandlerErrorMessage::AuthMiddleware,
                code: StatusCode::UNAUTHORIZED,
                src: errors::new_error("one of authorisation or middleware failed"),
            });
        }

        Ok(())
    }
}
