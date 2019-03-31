use hyper::rt::{self, Future, Stream};
use hyper::Client;
use std::io::{self, Write};

struct Puller {
    // blobstore: u8,
    // address: String,
}

impl Puller {
    pub fn new(address: &str, blobstore: &str) -> Puller {
        Puller { }
    }

    /// Retrieves all of the contents associated with an image that
    /// sits on a registry.
    ///
    /// 1. pull manifest + parse manifest
    /// 2. concurrently: [ config, blobs... ]
    ///
    pub fn pull(&self, name: &str, reference: &str) {}

    /// Retrieves the manifest that describes all of the contents of
    /// the image.
    ///
    /// ```txt
    /// GET /v2/$name/manifests/$reference
    /// ```
    ///
    fn pull_manifest(&self) {}

    /// Retrieve an image resource referenced in the image manifest.
    ///
    /// ```txt
    /// GET /v2/$name/blobs/$digest
    /// ```
    ///
    fn pull_image_resource(&self) {}
}
