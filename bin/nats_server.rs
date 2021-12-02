// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

extern crate engine_backend;
extern crate utilities;

use std::sync::Arc;

use engine_backend::nats::BackendNatsServer;
use utilities::result::Result;
use utilities::setup::SharedSetup;

#[tokio::main]
async fn main() -> Result<()> {
    let setup = Arc::new(SharedSetup::new().await?);
    let server = BackendNatsServer::new(setup);
    server.listen().await
}
