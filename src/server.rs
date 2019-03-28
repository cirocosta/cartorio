extern crate hyper;

use hyper::rt::Future;
use hyper::service::service_fn_ok;
use hyper::{Body, Request, Response, Server};
use std::net::SocketAddr;

const PHRASE: &str = "Hello, World!";

fn handle_hello_world(_req: Request<Body>) -> Response<Body> {
    Response::new(Body::from(PHRASE))
}

pub fn serve(address: &str) {
    let addr: SocketAddr = address.parse().unwrap();
    let new_svc = || service_fn_ok(handle_hello_world);

    let server = Server::bind(&addr)
        .serve(new_svc)
        .map_err(|e| println!("server error: {}", e));

    println!("listening on {}", address);

    hyper::rt::run(server);
}
