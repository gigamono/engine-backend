use std::sync::Arc;

use natsio::Message;
use utilities::{
    errors::{self, HandlerError, HandlerErrorMessage, SystemError},
    http::StatusCode,
    natsio,
    result::HandlerResult,
    setup::SharedSetup,
};

use crate::{FileManager, SurlExecutor};

pub(crate) async fn run_surl(setup: Arc<SharedSetup>, msg: &Message) -> HandlerResult<()> {
    // Get config.
    let config = &setup.config;

    // Deserialize request.
    let payload = natsio::deserialize(&msg.data).map_err(internal_error)?;

    // Create file manager.
    let file_mgr = FileManager::new(&payload, config)
        .await
        .map_err(internal_error)?;

    // Create surl runner.
    let surl_exec = SurlExecutor::new(file_mgr).await.map_err(internal_error)?;

    // Execute surl.
    if !surl_exec.execute().await.map_err(internal_error)? {
        // If result is false, then one of auth or middleware failed.
        return Err(HandlerError::Client {
            ctx: HandlerErrorMessage::AuthMiddleware,
            code: StatusCode::UNAUTHORIZED,
            src: errors::any_error("one of authorisation or middleware failed").unwrap_err(),
        });
    }

    // TODO(appcypher): Executing ths surl context should save response somewhere that we can then send or it should send it within the ops?

    // Publish message.
    msg.respond(msg.data.as_slice())
        .map_err(|err| HandlerError::Critical {
            src: errors::wrap_error("sending a reply", err).unwrap_err(),
        })
}

pub fn internal_error(err: SystemError) -> HandlerError {
    HandlerError::Internal {
        ctx: HandlerErrorMessage::InternalError,
        src: err,
    }
}
