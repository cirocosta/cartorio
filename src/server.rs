extern crate hyper;

use hyper::service::service_fn_ok;
use hyper::rt::Future;
use hyper::{Body, Request, Response, Server, Method, StatusCode};
use std::net::SocketAddr;

const BODY_NOT_FOUND: &str = "not found";

/// Handles incoming requests and dispatches them to the
/// appropriate function that is supposed to handle them.
///
fn handle_routing(req: Request<Body>) -> Response<Body> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/_live") => {
            let body = Body::from("alive");

           Response::builder()
                         .status(StatusCode::OK)
                         .body(body)
                         .unwrap()
        },

        (&Method::GET, "/v2") => {
            let body = Body::from("alive");

           Response::builder()
                         .status(StatusCode::OK)
                         .header("Docker-Distribution-Api-Version", "registry/2.0")
                         .body(body)
                         .unwrap()
        },

        _ => {
           Response::builder()
                         .status(StatusCode::NOT_FOUND)
                         .body(Body::from(BODY_NOT_FOUND))
                         .unwrap()
        }
    }
}

/// Starts an HTTP server at an address specified as `address`.
///
pub fn serve(address: &str) {
    let addr: SocketAddr = address.parse().unwrap();
    let server = Server::bind(&addr)
        .serve( || service_fn_ok(handle_routing))
        .map_err(|e| println!("server error: {}", e));

    println!("listening on {}", address);
    hyper::rt::run(server);
}
