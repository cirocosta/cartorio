use std::io;
use std::path::PathBuf;
use std::path::Path;

/// A filesystem-based store for the contents of images.
///
/// It manages the location where all of the files managed by
/// the registry are served from.
///
/// ```txt
/// (blobstore)
/// .
/// │
/// ├── bucket
/// │   ├── sha256:sha256(manifest_generated)
/// │   └── sha256:48e2eeb489cdea1578....0ecd34a
/// │
/// └── manifests
///     └── library
///         └─ nginx
///           ├── latest -> ../../bucket/sha256:sha256(manifest_generated)
///           └── sha256:sha256(manifest_generated) --> ../../bucket/sha256:sha256(manifest_generated)
/// ```
///
pub struct BlobStore {

    /// Where blobs exist 
    ///
    pub bucket_dir: PathBuf,

    /// Directory where manifests are put.
    ///
    pub manifests_dir: PathBuf,
}

impl BlobStore {

    /// Instantiates a blobstore - a place in the filesystem where all of
    /// the blobs associated with an image (as well as the manifest) exists.
    ///
    pub fn new(root: &Path) -> io::Result<BlobStore> {
        let blobstore = BlobStore {
            bucket_dir: Path::new(root).join("bucket"),
            manifests_dir: Path::new(root).join("manifests"),
        };

        std::fs::DirBuilder::new()
            .recursive(true)
            .create(&blobstore.bucket_dir)?;

        std::fs::DirBuilder::new()
            .recursive(true)
            .create(&blobstore.manifests_dir)?;

        Ok(blobstore)
    }


    /// Moves a blob to the store.
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
    fn move_blob(&self, blob_path: &Path) {
        unimplemented!("TBD");
    }

    /// Moves a manifest to the store.
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
    fn move_manifest(&self, manifest_path: &Path, tag: Option<&str>) {
        unimplemented!("TBD");
    }
}

