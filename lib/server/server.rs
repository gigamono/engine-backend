// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

use crate::{HttpDriver, Router};
use futures::{Future, FutureExt};
use log::{error, info};
use std::rc::Rc;
use std::thread;
use std::{panic::AssertUnwindSafe, sync::Arc};
use tokio::net::{TcpListener, TcpStream};
use tokio::runtime::Builder;
use tokio::sync::mpsc::{self, Sender};
use tokio::task::LocalSet;
use utilities::http::{self, Body, Request, Response};
use utilities::ip;
use utilities::result::HandlerResult;
use utilities::{
    result::{Context, Result},
    setup::CommonSetup,
};

pub struct BackendServer {
    setup: Arc<CommonSetup>,
}

impl BackendServer {
    pub fn new(setup: Arc<CommonSetup>) -> Self {
        Self { setup }
    }

    pub async fn listen(&self) -> Result<()> {
        // Initialize logger.
        env_logger::init();

        // Get socket address.
        let addr = ip::parse_socket_address(&self.setup.config.engines.backend.socket_address)?;

        info!(r#"Socket address = "{}""#, addr);

        // Bind to address.
        let tcp_listener = TcpListener::bind(addr).await.unwrap();

        // Accept client connections infinitely
        loop {
            // Handle connection and catch panics.
            self.connection_panic_wrap(Self::accept_connection, &tcp_listener)
                .await
        }
    }

    async fn accept_connection(&self, tcp_listener: &TcpListener) {
        // Accept client connection.
        let (tcp_stream, _) = tcp_listener.accept().await.unwrap();

        // Clone setup object.
        let setup = Arc::clone(&self.setup);

        // Spawn a thread for each client connection.
        thread::spawn(move || {
            // Create a thread local tokio runtime.
            let tokio_rt = Builder::new_current_thread()
                .build()
                .context("creating a new tokio runtime")
                .unwrap();

            // Create a local task set to run tasks on current thread because V8 Isolate (and some other objects) are !Send.
            let local = LocalSet::new();

            // Start handler task in new runtime.
            local.block_on(&tokio_rt, Self::handler(tcp_stream, setup, &local));
        });
    }

    async fn handler(tcp_stream: TcpStream, setup: Arc<CommonSetup>, local: &LocalSet) {
        // Request channel.
        let (request_tx, mut request_rx) = mpsc::channel(2);

        // Response Channel.
        let (response_tx, response_rx) = mpsc::channel(2);

        // Spawn task on local thread.
        local.spawn_local(HttpDriver::drive(
            tcp_stream,
            request_tx.clone(),
            response_rx,
        ));

        // Route and handle request if there is one.
        if let Some(request) = request_rx.recv().await {
            Self::handler_error_wrap(Router::route, request, response_tx, setup).await;
            return;
        }

        error!("No request to handle");
        unreachable!()
    }

    #[inline]
    async fn connection_panic_wrap<'a, 'b, F, Fut>(&'a self, func: F, tcp_listener: &'b TcpListener)
    where
        F: FnOnce(&'a Self, &'b TcpListener) -> Fut,
        Fut: Future<Output = ()>,
    {
        if let Err(err) = AssertUnwindSafe(func(self, tcp_listener))
            .catch_unwind()
            .await
        {
            let _ = http::handle_panic_error_t::<()>(err);
        }
    }

    #[inline]
    async fn handler_error_wrap<F, Fut>(
        func: F,
        request: Request<Body>,
        response_tx: Sender<Response<Body>>,
        setup: Arc<CommonSetup>,
    ) where
        F: FnOnce(Request<Body>, Rc<Sender<Response<Body>>>, Arc<CommonSetup>) -> Fut,
        Fut: Future<Output = HandlerResult<()>>,
    {
        let response_tx = Rc::new(response_tx);
        match func(request, Rc::clone(&response_tx), setup).await {
            Ok(_) => (),
            Err(err) => {
                // Log error.
                error!("{:?}", err.system_error());

                // Send handler error if possible.
                if let Err(err) = response_tx.send(err.as_hyper_response()).await {
                    error!("{:?}", err);
                };
            }
        }
    }
}
