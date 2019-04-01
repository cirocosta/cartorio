use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

struct BlobStore {
    bucket_dir: PathBuf,
    manifests_dir: PathBuf,
}

impl BlobStore {

    /// Instantiates a blobstore - a place in the filesystem where all of
    /// the blobs associated with an image (as well as the manifest) exists.
    ///
    fn new(root: &str) -> BlobStore {
        BlobStore {
            bucket_dir: Path::new(root).join("bucket"),
            manifests_dir: Path::new(root).join("manifests"),
        }
    }

    /// Adds a blob to the store.
    ///
    /// ```txt
    ///         .
    ///         ├── blobstore
    ///         │   ├── bucket
    ///         │   └── manifests
    ///         └── foo
    ///             └── layer.tar
    ///
    ///
    ///  add_blob("./foo/layer.tar")
    ///     -- possibly computes digest + moves with the right file name
    ///         -- digest might be in xattrs
    ///
    ///         .
    ///         ├── blobstore
    ///         │   ├── bucket
    ///         │   │   └── sha256:4bc453b53
    ///         │   └── manifests
    ///         └── foo
    ///
    /// ```
    ///
    /// # Arguments
    ///
    /// * `blob_path` - path to the blob file in the filesystem.
    ///
    fn add_blob(&self, blob_path: Path) {
        unimplemented!("TBD");
    }

    /// Adds a manifest to the store.
    ///
    /// ```txt
    ///
    ///         .
    ///         ├── blobstore
    ///         │   ├── bucket
    ///         │   └── manifests
    ///         │       └── latest
    ///         └── foo
    ///             └── manifest.json
    ///
    ///
    ///  add_manifest("./foo/manifest.json", "latest")
    ///     -- (possibly computes digest + performs moves + links)
    ///             -- digest might be in xattrs
    ///
    ///         .
    ///         ├── blobstore
    ///         │   ├── bucket
    ///         │   │   └── sha256:4bc453b5
    ///         │   └── manifests
    ///         │       ├── latest -> ../bucket/sha256:4bc453
    ///         │       └── sha256:4bc453b -> ../bucket/sha256:4bc453b53
    ///         └── foo
    ///
    /// ```
    ///
    /// # Arguments
    ///
    /// * `manifest_path` - location on disk where the manifest file exists.
    /// * `tag` - an optional tag for such manifest.
    ///
    fn add_manifest(&self, manifest_path: Path, tag: Option<&str>) {
        unimplemented!("TBD");
    }
}

