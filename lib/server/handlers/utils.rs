use std::sync::Arc;

use utilities::{natsio::Message, result::{Result, Context}};

pub(crate) fn get_first_from_headers(msg: &Arc<Message>, key: impl AsRef<str>) -> Result<String> {
    let headers = msg.headers.as_ref().unwrap().get(key.as_ref());
    let values = headers.unwrap();
    let value = values.iter().next().unwrap();

    Ok(value.clone())
}
