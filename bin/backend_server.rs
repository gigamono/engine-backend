// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

extern crate engine_backend;
extern crate utilities;

use std::sync::Arc;

use engine_backend::BackendServer;
use utilities::result::Result;
use utilities::setup::CommonSetup;

#[tokio::main]
async fn main() -> Result<()> {
    let setup = Arc::new(CommonSetup::new().await?);
    let server = BackendServer::new(setup);
    server.listen().await
}
