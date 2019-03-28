extern crate hyper;

use hyper::rt::Future;
use hyper::service::service_fn_ok;
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use std::net::SocketAddr;

const BODY_NOT_FOUND: &str = "not found";

/// Handles incoming requests and dispatches them to the
/// appropriate function that is supposed to handle them.
///
fn handle_routing(req: Request<Body>) -> Response<Body> {
    let method = req.method();
    let path = req.uri().path();

    if method == &Method::GET && path == "/_live" {
        return Response::builder()
            .status(StatusCode::OK)
            .body(Body::from("alive"))
            .unwrap()

    } else if method == &Method::GET && path == "/v2"  {
        return Response::builder()
            .status(StatusCode::OK)
            .header("docker-distribution-api-version", "registry/2.0")
            .body(Body::empty())
            .unwrap()
    }

    Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from(BODY_NOT_FOUND))
                .unwrap()
}

/// Represents a manifest path
struct ManifestPath {
    Name: String,
    Reference: String,
}

/// Verifies whether we're looking at a path that is
/// destined to the `v2` routes.
///
fn parse_manifests_path(path: &str) -> Option<ManifestPath> {
    if !path.starts_with("/v2") {
        return None;
    }

    Some(ManifestPath{
        Name: "test".to_string(),
        Reference: "test".to_string(),
    })
}

//     // check if it starts with `/v2`
// }

/// Verifies whether we're looking at a path that is
/// destined to the `v2` routes.
///
// fn is_registry_path(path: &str) -> bool {
//     // check if it starts with `/v2`
// }

/// Starts an HTTP server at an address specified as `address`.
///
pub fn serve(address: &str) {
    let addr: SocketAddr = address.parse().unwrap();
    let server = Server::bind(&addr)
        .serve(|| service_fn_ok(handle_routing))
        .map_err(|e| println!("server error: {}", e));

    println!("listening on {}", address);
    hyper::rt::run(server);
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_manifests_path() {
        assert!(parse_manifests_path("xxx").is_none());
    }
}

