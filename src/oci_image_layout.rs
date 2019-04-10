use std::str::FromStr;
use std::path::Path;
use std::collections::HashMap;

use serde::Deserialize;

use crate::blobstore::BlobStore;
use crate::error::Result;



/// Platform information associated with a manifest.
///
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct OciManifestPlatform {
    architecture: String,
    os: String,
}


/// The pointer to a single manifest that exists within an OCI
/// Image Index.
///
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct OciImageIndexManifest {
    media_type: String,
    digest: String,
    size: u64,
    annotations: HashMap<String, String>,
    platform: OciManifestPlatform,
}


/// The image index acts as a pointer to manifests that can be
/// found under the `blobs` directory, providing not only the
/// content-addressable identifier of such manifests, but also
/// extra information associated with them (e.g., platform and
/// annotations).
///
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct OciImageIndex {
    schema_version: u8,
    manifests: Vec<OciImageIndexManifest>,
}


/// Represents an OCI Image Layout directory.
///
/// For instance:
///
/// ```txt
///
///     .
///     ├── blobs
///     │   └── sha256
///     │       ├── 5fffaf1f2c18...
///     │       ├── dbaff2215b8b...
///     │       └── fc1a6b909f82...
///     ├── index.json                  == OciImageIndex
///     └── oci-layout
///
/// ```
///
/// Where:
///
/// * `oci-layout` - presents information about the layout of this directory
/// * `index.json` - lists manifests 
/// * `blobs/sha256/*` - content referenced by the OciImageIndex and the manifests
///
///
/// References:
///
/// - [OCI Image Layout](https://github.com/opencontainers/image-spec/blob/master/image-layout.md)
///
pub struct OciImageLayout {

    /// The final owner of the blobs and manifests for the
    /// registry to serve.
    ///
    /// [`BlobStore]: struct.BlobStore.html
    ///
    blobstore: BlobStore,

}


impl OciImageLayout {

    /// Instantiates a new OciImageLayout object, validating that the directory that
    /// it should reference is indeed valid.
    ///
    /// # Arguments
    ///
    /// * `dir` - location where the OCI Image Layout directory exists
    /// * `name` - name of the image
    /// * `blobstore` - a BlobStore to own the images from such directory.
    ///
    pub fn new(dir: &Path, name: &str, blobstore: BlobStore) -> Result<OciImageLayout> {
        unimplemented!();
    }


    /// Loads the contents found in an OCI Image Layout directory into the blobstore.
    ///
    ///
    /// ```txt
    ///
    ///    .
    ///    ├── blobs
    ///    │   └── sha256
    ///    │       ├── 5fffaf1f2c18...
    ///    │       ├── dbaff2215b8b...
    ///    │       └── fc1a6b909f82...
    ///    ├── index.json                  == OciImageIndex
    ///    └── oci-layout
    ///
    ///
    /// ==>
    ///
    ///
    ///    .
    ///    ├── bucket
    ///    │   ├── sha256:5fffaf1f2c18...
    ///    │   ├── sha256:dbaff2215b8b...
    ///    │   └── sha256:fc1a6b909f82...
    ///    │
    ///    └── manifests
    ///        └── library
    ///            └─ $name
    ///              ├── latest -> ../../bucket/sha256:dbafff2...
    ///              └── sha256:dbafff2... --> ../../bucket/dbafff2...
    ///
    /// ```
    ///
    ///
    pub fn load(&self) -> Result<()> {
        // move all blobs over to the blobstore
        // parse index.json (OciImageIndex)
        // for each manifest:
        //    tag accordingly

        unimplemented!();
    }

}
