// Copyright 2021 the Gigamono authors. All rights reserved. GPL-3.0 License.

extern crate engine_runtime;
extern crate utilities;

use std::sync::Arc;

use engine_runtime::RuntimeServer;
use utilities::result::Result;
use utilities::setup::CommonSetup;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logger.
    env_logger::init();

    let setup = Arc::new(CommonSetup::new().await?);
    let server = RuntimeServer::new(setup);
    server.listen().await
}
