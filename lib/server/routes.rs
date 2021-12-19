// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

use crate::handlers::ApiHandler;
use std::{rc::Rc, sync::Arc};
use tokio::sync::mpsc::Sender;
use utilities::{
    errors::{self, HandlerError, HandlerErrorMessage},
    hyper::{Body, Request, Response, StatusCode},
    result::HandlerResult,
    setup::CommonSetup,
};

pub struct Router;

impl Router {
    pub async fn route(
        request: Request<Body>,
        response_tx: Rc<Sender<Response<Body>>>,
        setup: Arc<CommonSetup>,
    ) -> HandlerResult<()> {
        let path = request.uri().path();

        // Routing.
        if path.starts_with("/api/") {
            ApiHandler::handle(request, response_tx, setup).await
        } else {
            Err(HandlerError::Client {
                ctx: HandlerErrorMessage::NotFound,
                code: StatusCode::NOT_FOUND,
                src: errors::new_error(format!(r#"resource not found "{}""#, path)),
            })
        }
    }
}
