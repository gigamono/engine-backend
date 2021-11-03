use std::sync::Arc;

use natsio::Message;
use utilities::{
    http::StatusCode,
    messages::error::{HandlerError, HandlerErrorMessage, SystemError},
    natsio,
    result::HandlerResult,
    setup::SharedSetup,
};

use crate::{FileManager, SurlContext};

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
    let ctx = SurlContext::new(file_mgr).await.map_err(internal_error)?;

    // Execute surl.
    if ctx.execute().await.map_err(internal_error)? {
        // If result is false, then one of auth or middleware failed.
        return Err(HandlerError::Client {
            ctx: HandlerErrorMessage::AuthMiddleware,
            code: StatusCode::UNAUTHORIZED,
            src: SystemError::Generic {
                ctx: "one of authorisation or middleware failed".to_string(),
            },
        });
    }

    // TODO: Executing ths surl context should save response somewhere that we can then send.
    //       Or it should send it within the ops?

    // Publish message.
    msg.respond(msg.data.as_slice())
        .map_err(|err| HandlerError::Critical {
            src: SystemError::Io {
                ctx: "sending a reply".to_string(),
                src: err,
            },
        })
}

pub fn internal_error(err: SystemError) -> HandlerError {
    HandlerError::Internal {
        ctx: HandlerErrorMessage::InternalError,
        src: err,
    }
}
