// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

use crate::{files::FileManager, runtimes::ApiRuntime};
use std::{cell::RefCell, rc::Rc, sync::Arc};
use tera::events::{Events, HttpEvent, HttpResponder};
use tokio::sync::mpsc::Sender;
use utilities::{
    errors::{self, HandlerError, HandlerErrorMessage},
    http,
    hyper::{Body, Request, Response, StatusCode},
    result::HandlerResult,
    setup::CommonSetup,
};

pub struct ApiHandler;

impl ApiHandler {
    pub async fn handle(
        request: Request<Body>,
        response_tx: Rc<Sender<Response<Body>>>,
        setup: Arc<CommonSetup>,
    ) -> HandlerResult<()> {
        // Get config.
        let config = &setup.config;

        // Get workspace id.
        let workspace_id = http::get_header_value(&request, http::WORKSPACE_ID_HEADER)
            .map_err(http::internal_error)?;

        // Get url path.
        let url_path = request.uri().path();

        // Create file manager.
        let file_mgr = FileManager::new(&workspace_id, &url_path, config)
            .await
            .map_err(http::internal_error)?;

        // Events.
        let events = Rc::new(RefCell::new(Events {
            http: Some(HttpEvent::new(
                request,
                Rc::new(HttpResponder::new(response_tx)),
            )),
        }));

        // Create api runtime.
        let api_rt = ApiRuntime::new(file_mgr, events)
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
