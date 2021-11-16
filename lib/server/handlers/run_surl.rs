use std::{str::Bytes, sync::Arc};

use natsio::Message;
use utilities::{
    errors::{self, HandlerError, HandlerErrorMessage, SystemError},
    http::StatusCode,
    natsio,
    result::HandlerResult,
    setup::SharedSetup,
};

use crate::{FileManager, SurlExecutor};

pub(crate) async fn run_surl(setup: Arc<SharedSetup>, msg: Arc<Message>) -> HandlerResult<Vec<u8>> {
    // Get config.
    let config = &setup.config;

    // Get workspace id and URL path from message headers.
    let workspace_id = natsio::get_first_from_headers(&msg, natsio::WORKSPACE_ID_HEADER)
        .map_err(internal_error)?;

    let url_path =
        natsio::get_first_from_headers(&msg, natsio::URL_PATH_HEADER).map_err(internal_error)?;

    // Create file manager.
    let file_mgr = FileManager::new(&workspace_id, &url_path, config)
        .await
        .map_err(internal_error)?;

    // Create surl runner.
    let surl_exec = SurlExecutor::new(file_mgr).await.map_err(internal_error)?;

    // Execute surl.
    if !surl_exec.execute().await.map_err(internal_error)? {
        // If result is false, then one of auth or middleware failed.
        return Err(HandlerError::Client {
            ctx: HandlerErrorMessage::AuthMiddleware,
            code: StatusCode::Unauthorized,
            src: errors::any_error("one of authorisation or middleware failed").unwrap_err(),
        });
    }

    // TODO(appcypher): Executing ths surl context should save response somewhere that we can then send or it should send it within the ops?\

    // Return message.
    // TODO(appcypher): Return bytes
    Ok(vec![])
}

pub fn internal_error(err: SystemError) -> HandlerError {
    HandlerError::Internal {
        ctx: HandlerErrorMessage::InternalError,
        src: err,
    }
}
