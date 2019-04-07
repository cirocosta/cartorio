extern crate serde_json;

use crate::digest;
use crate::registry::Manifest;

use std::fs::DirBuilder;
use std::io;
use std::io::Write;
use std::path::PathBuf;
use std::path::Path;
use std::os::unix::fs::symlink;


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
    /// * `blob` - path to the blob file in the filesystem.
    ///
    pub fn add_blob(&self, blob: &Path) -> io::Result<()> {
        let blob_digest = digest::retrieve_or_compute_and_store(blob)?;
        let blob_filename = digest::prepend_sha_scheme(&blob_digest);
        let blob_bucket_path = self.bucket_dir.join(blob_filename);

        std::fs::rename(
            blob,
            blob_bucket_path,
        )
    }

    /// Writes an image manifest to the store.
    ///
    /// Given a [`Manifest`], this method will serialize the struct into
    /// JSON, compute its digest and then write it to the bucket of blobs.
    ///
    ///
    /// # Arguments
    ///
    /// * `manifest` - the manifest to persist.
    ///
    pub fn add_manifest(&self, manifest: &Manifest) -> io::Result<String> {

        let manifest_json = serde_json::to_string_pretty(&manifest).unwrap();
        let manifest_json_digest = digest::compute_for_string(&manifest_json);
        let manifest_filename = digest::prepend_sha_scheme(&manifest_json_digest);
        let manifest_bucket_path = self.bucket_dir.join(&manifest_filename);

        let mut manifest_file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(&manifest_bucket_path)
            .unwrap();

        manifest_file
            .write_all(manifest_json.as_bytes())?;

        digest::store(
            &manifest_bucket_path, 
            &manifest_json_digest,
        )?;

        Ok(manifest_filename)
    }


    /// Links a manifest to a blob that represents it.
    ///
    /// ```txt
    ///
    ///         .
    ///         └── blobstore
    ///             ├── bucket
    ///             │   └── sha256:4bc453b5
    ///             └── manifests
    ///
    ///
    ///  tag_manifest("sha256:4bc453b", "name", "latest")
    ///
    ///         .
    ///         ├── blobstore
    ///         │   ├── bucket
    ///         │   │   └── sha256:4bc453b5
    ///         │   └── manifests
    ///         │       └── name
    ///         │           └── latest -> ../bucket/sha256:4bc453
    ///         └── foo
    ///
    /// ```
    ///
    /// # Arguments
    ///
    /// * `filename` - name of the digest file under `bucket_dir`.
    /// * `name` - name of the image
    /// * `reference` - reference (either digest or tag).
    ///
    pub fn tag_manifest(&self, filename: &str, name: &str, reference: &str) -> io::Result<()> {
        let manifest_bucket_path = self.bucket_dir.join(filename);

        DirBuilder::new()
            .recursive(true)
            .create(self.manifests_dir.join(filename))?;

        symlink(
            &manifest_bucket_path,
            self.manifests_dir
                .join(name)
                .join(reference),
        );

        Ok(())
    }

}

