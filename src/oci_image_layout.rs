use std::str::FromStr;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::fs;

use serde::Deserialize;

use crate::blobstore::BlobStore;
use crate::digest;
use crate::error::Result;



/// Platform information associated with a manifest.
///
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct OciManifestPlatform {
    architecture: String,
    os: String,
}


/// The pointer to a single manifest that exists within an OCI
/// Image Index.
///
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct OciImageIndexManifest {
    media_type: String,
    digest: String,
    size: u64,
    annotations: HashMap<String, String>,
    platform: OciManifestPlatform,
}


/// Representation of an OCI Image Index.
///
/// The image index acts as a pointer to manifests that can be
/// found under the `blobs` directory, providing not only the
/// content-addressable identifier of such manifests, but also
/// extra information associated with them (e.g., platform and
/// annotations).
///
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OciImageIndex {
    schema_version: u8,
    manifests: Vec<OciImageIndexManifest>,
}

impl FromStr for OciImageIndex {

    type Err = serde_json::Error;

    fn from_str(content: &str) -> std::result::Result<Self, Self::Err> {
        let image_index: OciImageIndex = serde_json::from_str(content)?;

        Ok(image_index)
    }

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

    /// The representation of the OCI Image Index found at the OCI Image Layout
    /// directory.
    ///
    image_index: OciImageIndex,

    /// Root directory of the image layout.
    ///
    root_dir: PathBuf,

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
        let index_content = fs::read_to_string(dir.join("index.json"))?;

        let image_index: OciImageIndex = index_content.parse()?;

        Ok(OciImageLayout{
            blobstore: blobstore,
            image_index: image_index,
            root_dir: dir.to_owned(),
        })
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
        let blob_contents_dir = self.root_dir.join("blobs").join("sha256");

        for entry in fs::read_dir(blob_contents_dir)? {
            let blob = entry.unwrap();

            self.blobstore.add_blob_with_digest(
                blob.path().as_ref(),
                blob.file_name().to_str().unwrap(),
            )?;
        }

        for manifest in &self.image_index.manifests {
            let manifest_name = digest::prepend_sha_scheme(&manifest.digest);
            let manifest_path = self.blobstore.get_blob(&manifest_name);

            // tagging digest to the digest
            self.blobstore.tag_manifest(&manifest_name, "test", &manifest_name)?;
        }


        // TAGGING
        //
        //    name: supplied through `new`
        //    filename: we know from the index
        //    reference:
        //     - for the digest: we know from the index;
        //     - for tag: we need to parse the properties map
        //


        Ok(())
    }

}
