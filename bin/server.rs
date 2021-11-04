extern crate engine_backend;
extern crate utilities;

use std::sync::Arc;

use engine_backend::BackendServer;
use utilities::result::Result;
use utilities::setup::SharedSetup;

#[tokio::main]
async fn main() -> Result<()> {
    let setup = Arc::new(SharedSetup::new()?);
    let server = BackendServer::new(setup);
    server.handle_subscriptions().await
}
