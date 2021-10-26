use std::sync::Arc;

use natsio::Message;
use utilities::{
    messages::error::{HandlerError, HandlerErrorMessage, SystemError},
    natsio,
    result::HandlerResult,
    setup::SharedSetup,
};

use crate::{executor_surl::SurlExecutor, file_manager::FileManager};

pub(crate) async fn run_surl(setup: Arc<SharedSetup>, msg: &Message) -> HandlerResult<()> {
    // Get config.
    let config = &setup.config;

    // Deserialize request.
    let payload = natsio::deserialize(&msg.data).map_err(internal_error)?;

    // Create file manager.
    let file_mgr = FileManager::new(&payload, config)
        .await
        .map_err(internal_error)?;

    // Create executor.
    let executor = SurlExecutor::new(file_mgr).await.map_err(internal_error)?;

    // TODO: Do something result.
    let result = executor.execute().await.map_err(internal_error)?;

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
