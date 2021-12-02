// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

use std::pin::Pin;
use tera::events::HttpEventStreamer;
use utilities::{http::Response, result::Result};

pub struct BackendStreamer {}

impl BackendStreamer {
    pub fn new() -> Self {
        Self {}
    }
}

impl HttpEventStreamer for BackendStreamer {
    fn read_request_body(&self) -> Pin<Box<Result<Vec<u8>>>> {
        todo!()
    }

    fn write_response_body(&self, response: Response) -> Pin<Box<Result<()>>> {
        todo!()
    }
}
