use std::net::SocketAddr;

use futures::Future;
use futures_fs::FsPool;
use hyper::service::service_fn_ok;
use hyper::{Body, Method, Request, Response, Server, StatusCode};

use crate::blobstore::BlobStore;


const BODY_NOT_FOUND: &str = "not found";


/// Represents a manifest path.
///
struct BlobPath {
    name: String,
    reference: String,
}


/// Parses a path into a BlobPath.
///
fn parse_generic_blob_path(path_type: &'static str, path: &str) -> Option<BlobPath> {
    let splitted: Vec<&str> = path.trim_matches('/').split("/").collect();

    if splitted.len() < 4 {
        return None;
    }

    if splitted[0] != "v2" {
        return None;
    }

    if splitted[splitted.len() - 2] != path_type {
        return None;
    }

    let reference = splitted[splitted.len() - 1];
    let name = &splitted[1..splitted.len() - 2];

    Some(BlobPath {
        name: name.join("/"),
        reference: reference.to_string(),
    })
}


/// Detects whether the provided `path` is a `BlobPath` and,
/// if so, returns a `BlobPath`.
///
fn parse_manifests_path(path: &str) -> Option<BlobPath> {
    parse_generic_blob_path("manifests", path)
}


fn parse_blobs_path(path: &str) -> Option<BlobPath> {
    parse_generic_blob_path("blobs", path)
}


/// Starts an HTTP server for serving the registry's content.
///
/// # Arguments
///
/// * `address` - IPV4 address to bind to listen for requests
///
/// See `loader`.
///
pub fn serve(address: &str, blobstore: BlobStore) {
    let addr: SocketAddr = address.parse().unwrap();

    let routing_svc = move || {
        let bstore = blobstore.clone();

        service_fn_ok(move |req| {
            if let Some(resp) = handle_liveness_check(&req) {
                return resp;
            } else if let Some(resp) = handle_registry_version_check(&req) {
                return resp;
            } else if let Some(resp) = handle_registry_manifests(&req, &bstore) {
                return resp;
            } else if let Some(resp) = handle_registry_blobs(&req, &bstore) {
                return resp;
            }

            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from(BODY_NOT_FOUND))
                .unwrap()
        })
    };

    let server = Server::bind(&addr)
        .serve(routing_svc)
        .map_err(|e| println!("server error: {}", e));

    println!("listening on http://{}", address);

    hyper::rt::run(server);
}


/// Handles blob requests.
///
fn handle_registry_blobs(req: &Request<Body>, blobstore: &BlobStore) -> Option<Response<Body>> {
    if req.method() != &Method::GET {
        return None;
    }

    let blob_info = parse_blobs_path(req.uri().path())?;

    let file_path = blobstore
        .get_blob(&blob_info.reference);

    let file = FsPool::default()
        .read(file_path, Default::default());

    Some(
        Response::builder()
            .header("content-type", "application/octet-stream")
            .header("docker-content-digest", blob_info.reference.as_bytes())
            .header("etag", blob_info.reference.as_bytes())
            .header("docker-distribution-api-version", "registry/2.0")
            .status(StatusCode::OK)
            .body(Body::wrap_stream(file))
            .unwrap(),
    )
}


/// Handles requests for manifests.
///
/// ```txt
/// GET /v2/foo/bar/manifests/tag
/// ```
///
fn handle_registry_manifests(req: &Request<Body>, blobstore: &BlobStore) -> Option<Response<Body>> {
    if req.method() != &Method::GET {
        return None;
    }

    let manifest_info = parse_manifests_path(req.uri().path())?;

    let manifest_file_path = match std::fs::read_link(
        blobstore.get_manifest(
            &manifest_info.name,
            &manifest_info.reference,
        ),
    ) {
        Err(err) => {
            panic!("oh damn, it probably doesn't exist - {}", err);
        },

        Ok(fp) => fp,
    };

    let manifest_digest = manifest_file_path
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();

    let file = FsPool::default().read(
        manifest_file_path,
        Default::default(),
    );

    Some(
        Response::builder()
            .header(
                "content-type",
                "application/vnd.docker.distribution.manifest.v2+json",
            )
            .header("docker-content-digest", manifest_digest.as_bytes())
            .header("etag", manifest_digest.as_bytes())
            .header("docker-distribution-api-version", "registry/2.0")
            .status(StatusCode::OK)
            .body(Body::wrap_stream(file))
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



#[cfg(test)]
mod parsing_tests {
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
