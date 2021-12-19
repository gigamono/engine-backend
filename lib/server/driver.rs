// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

use log::error;
use std::{cell::RefCell, convert::Infallible, rc::Rc};
use tokio::{net::TcpStream, sync::mpsc};
use utilities::{
    errors::{self},
    http,
    hyper::{rt::Executor, server::conn::Http, service::service_fn, Body, Request, Response},
};

#[derive(Clone)]
struct LocalExecutor;

pub struct HttpDriver;

impl HttpDriver {
    pub async fn drive(
        tcp_stream: TcpStream,
        request_tx: mpsc::Sender<Request<Body>>,
        response_rx: mpsc::Receiver<Response<Body>>,
    ) {
        let response_rx = Rc::new(RefCell::new(response_rx));

        // Set up http handling context.
        Http::new()
            .with_executor(LocalExecutor)
            .serve_connection(
                tcp_stream,
                service_fn(move |request| {
                    let request_tx = request_tx.clone();
                    let response_rx = Rc::clone(&response_rx);

                    async move {
                        let mut response_rx = response_rx.borrow_mut();

                        let mut response =
                            http::internal_error(errors::new_error("")).as_hyper_response();

                        // Send request.
                        if let Err(err) = request_tx.send(request).await {
                            error!("{:?}", err);
                            return Ok(response);
                        }

                        // Wait for response.
                        if let Some(resp) = response_rx.recv().await {
                            response = resp;
                        } else {
                            error!("no response recieved");
                        }

                        Ok::<_, Infallible>(response)
                    }
                }),
            )
            .await
            .expect("serving connection");
    }
}

impl<F> Executor<F> for LocalExecutor
where
    F: std::future::Future + 'static,
{
    fn execute(&self, fut: F) {
        // This will spawn into the currently running `LocalSet`.
        tokio::task::spawn_local(fut);
    }
}
