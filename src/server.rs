extern crate hyper;

use hyper::rt::Future;
use hyper::service::service_fn_ok;
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use std::net::SocketAddr;

const BODY_NOT_FOUND: &str = "not found";

/// Represents a manifest path.
///
struct ManifestPath {
    name: String,
    reference: String,
}

/// Detects whether the provided `path` is a `ManifestPath` and,
/// if so, returns a `ManifestPath`.
///
fn parse_manifests_path(path: &str) -> Option<ManifestPath> {
    let splitted: Vec<&str> = path.trim_matches('/').split("/").collect();

    if splitted.len() < 4 {
        return None;
    }

    if splitted[0] != "v2" {
        return None;
    }

    if splitted[splitted.len() - 2] != "manifests" {
        return None;
    }

    let reference = splitted[splitted.len() - 1];
    let name = &splitted[1..splitted.len() - 2];

    Some(ManifestPath {
        name: name.join("/"),
        reference: reference.to_string(),
    })
}

fn handle_registry_manifests(req: &Request<Body>) -> Option<Response<Body>> {
    if req.method() != &Method::GET {
        return None;
    }

    let manifestPath = match parse_manifests_path(req.uri().path()) {
        Some(m) => m,
        _ => return None,
    };

    Some(
        Response::builder()
            .status(StatusCode::OK)
            .body(Body::from("unimplemented yet"))
            .unwrap(),
    )
}

fn handle_registry_version_check(req: &Request<Body>) -> Option<Response<Body>> {
    if req.method() != &Method::GET || req.uri().path() != "/v2" {
        return None;
    }

    Some(
        Response::builder()
            .status(StatusCode::OK)
            .header("docker-distribution-api-version", "registry/2.0")
            .body(Body::empty())
            .unwrap(),
    )
}

fn handle_liveness_check(req: &Request<Body>) -> Option<Response<Body>> {
    if req.method() != &Method::GET || req.uri().path() != "/_live" {
        return None;
    }

    Some(
        Response::builder()
            .status(StatusCode::OK)
            .body(Body::from("alive"))
            .unwrap(),
    )
}

/// Handles incoming requests and dispatches them to the
/// appropriate function that is supposed to handle them.
///
fn route(req: Request<Body>) -> Response<Body> {
    if let Some(resp) = handle_liveness_check(&req) {
        return resp;
    } else if let Some(resp) = handle_registry_version_check(&req) {
        return resp;
    } else if let Some(resp) = handle_registry_manifests(&req) {
        return resp;
    }

    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::from(BODY_NOT_FOUND))
        .unwrap()
}

/// Starts an HTTP server at an address specified as `address`.
///
pub fn serve(address: &str) {
    let handlers: &[&Fn(&Request<Body>) -> Option<Response<Body>>] = &[&handle_liveness_check];

    let addr: SocketAddr = address.parse().unwrap();

    let server = Server::bind(&addr)
        .serve(|| service_fn_ok(route))
        .map_err(|e| println!("server error: {}", e));

    println!("listening on {}", address);
    hyper::rt::run(server);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_manifests_path() {
        assert!(
            parse_manifests_path("xxx").is_none(),
            "must have a `/v2` in the prefix`"
        );

        assert!(
            parse_manifests_path("/v2/library/manifests").is_none(),
            "must have enough fields"
        );

        assert!(
            parse_manifests_path("/v2/library/wrong/tag").is_none(),
            "must have `manifests` after name and before reference"
        );

        assert_eq!(
            parse_manifests_path("/v2/library/manifests/tag")
                .unwrap()
                .name,
            "library",
        );

        assert_eq!(
            parse_manifests_path("/v2/library/manifests/tag")
                .unwrap()
                .reference,
            "tag",
        );

        assert_eq!(
            parse_manifests_path("/v2/library/nginx/manifests/tag")
                .unwrap()
                .name,
            "library/nginx",
        );

        assert_eq!(
            parse_manifests_path("/v2/library/nginx/manifests/sha256:7422e18d69adca5354c08f92dd18192fa142eda4cc891d093f22edbb38c4de1b")
                .unwrap()
                .reference,
            "sha256:7422e18d69adca5354c08f92dd18192fa142eda4cc891d093f22edbb38c4de1b",
        );
    }
}
