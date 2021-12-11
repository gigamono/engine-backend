// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

use log::{debug, error};
use std::{cell::RefCell, convert::Infallible, rc::Rc};
use tokio::{net::TcpStream, sync::mpsc};
use utilities::http::{Body, Request, Response, rt::Executor, server::conn::Http, service::service_fn};

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
                service_fn(move |request: Request<Body>| {
                    let request_tx = request_tx.clone();
                    let response_rx = Rc::clone(&response_rx);

                    async move {
                        let mut response_rx = response_rx.borrow_mut();

                        // Get futures for sending request and recieving response.
                        let send_req_fut = request_tx.send(request);
                        let recv_resp_fut = response_rx.recv();

                        // TODO(appcyper): Return an internal server error.
                        let mut response = Response::default();
                        tokio::select! {
                            result = send_req_fut => {
                                if let Err(err) = result {
                                    error!("{:?}", err);
                                };

                                // Continue waiting for response
                                if let Some(resp) = response_rx.recv().await {
                                    response = resp;
                                }
                            }
                            result = recv_resp_fut => {
                                if let Some(resp) = result {
                                    response = resp;
                                }
                            }
                        }

                        Ok::<Response<Body>, Infallible>(response)
                    }
                }),
            )
            .await
            .expect("serving connection");

        debug!("Returning the server!");
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
