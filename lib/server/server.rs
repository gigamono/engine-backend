// Copyright 2021 the Gigamono authors. All rights reserved. GPL-3.0 License.

use crate::{HttpDriver, Router};
use futures::{Future, FutureExt};
use log::{error, info};
use std::rc::Rc;
use std::thread;
use std::{panic::AssertUnwindSafe, sync::Arc};
use tera::errors::JsError;
use tokio::net::{TcpListener, TcpStream};
use tokio::runtime::Builder;
use tokio::sync::mpsc::{self, Sender};
use tokio::task::LocalSet;
use utilities::errors::{self, HandlerError, HandlerErrorMessage};
use utilities::hyper::{Body, Request, Response, StatusCode};
use utilities::result::HandlerResult;
use utilities::{http, ip};
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

        // TODO(appcypher): Need hard or soft limit on thread spawn.
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
        let (request_tx, mut request_rx) = mpsc::channel(1);

        // Response Channel.
        let (response_tx, response_rx) = mpsc::channel(1);

        // Spawn task on local thread.
        local.spawn_local(HttpDriver::drive(
            tcp_stream,
            request_tx.clone(),
            response_rx,
        ));

        // Route and handle request if there is one.
        if let Some(request) = request_rx.recv().await {
            Self::handler_error_wrap(Router::route, request, response_tx, setup).await;
        }

        // This is here to prevent the runtime started by `local.block_on` from ending early before
        // the driver has properly handled the response we just sent.
        request_rx.recv().await.unwrap();
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
            Err(mut err) => {
                // Log error.
                error!("{:?}", err.system_error());

                // Customize js errors that are permission errors.
                Self::customize_permission_error(&mut err);

                // Send handler error.
                if let Err(err) = response_tx.try_send(err.as_hyper_response()) {
                    error!("{:?}", err);
                };
            }
        }
    }

    fn customize_permission_error(mut handler_err: &mut HandlerError) {
        if let HandlerError::Internal { src, .. } = &mut handler_err {
            if let Some(js_err) = src.downcast_ref::<JsError>() {
                if js_err.message.contains("CustomError::Permission") {
                    *handler_err = HandlerError::Client {
                        ctx: HandlerErrorMessage::AuthMiddleware,
                        code: StatusCode::UNAUTHORIZED,
                        src: errors::new_error("permission error from JavaScript land"),
                    };
                }
            }
        };
    }
}
