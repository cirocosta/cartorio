use std::fs::DirBuilder;
use std::io::Write;
use std::os::unix::fs::symlink;
use std::path::Path;
use std::path::PathBuf;

use crate::digest;
use crate::error::Result;
use crate::registry::Manifest;


/// A filesystem-based store for the contents of images.
///
/// It manages the location where all of the files managed by
/// the registry are served from.
///
/// ```txt
///
///    .
///    ├── bucket
///    │   ├── sha256:sha256(manifest_generated)
///    │   └── sha256:48e2eeb489cdea1578....0ecd34a
///    │
///    └── manifests
///        └── library
///            └─ nginx
///              ├── latest -> ../../bucket/sha256:sha256(manifest_generated)
///              └── sha256:sha256(manifest_generated) --> ../../bucket/sha256:sha256(manifest_generated)
/// ```
///
#[derive(Clone)]
pub struct BlobStore {

    /// Where blobs exist 
    ///
    pub bucket_dir: PathBuf,

    /// Directory where manifests are put.
    ///
    pub manifests_dir: PathBuf,
}


impl BlobStore {

    const BUCKET_DIR_NAME: &'static str = "bucket";
    const MANIFESTS_DIR_NAME: &'static str = "manifests";


    /// Instantiates a blobstore - a place in the filesystem where all of
    /// the blobs associated with an image (as well as the manifest) exists.
    ///
    pub fn new(root: &Path) -> Result<BlobStore> {

        let blobstore = BlobStore {
            bucket_dir: root.join(BlobStore::BUCKET_DIR_NAME),
            manifests_dir: root.join(BlobStore::MANIFESTS_DIR_NAME),
        };

        DirBuilder::new()
            .recursive(true)
            .create(&blobstore.bucket_dir)?;

        DirBuilder::new()
            .recursive(true)
            .create(&blobstore.manifests_dir)?;

        Ok(blobstore)
    }


    /// Retrieves the path in the filesystem to the desired blob.
    ///
    ///
    /// # Arguments
    ///
    /// * `name`: name of the blob with the digest scheme (e.g., `sha256:abcdef`).
    ///
    ///
    /// # Remarks
    ///
    /// This method WILL NOT check if the file exists or not as this would
    /// require making use of blocking syscalls.
    ///
    pub fn get_blob(&self, name: &str) -> PathBuf {
        self.bucket_dir.join(&name)
    }


    /// Retrieves the path in the filesystem to the desired blob.
    ///
    ///
    /// # Arguments
    ///
    /// * `name`: name of the blob (e.g., `sha256:abcdef`).
    ///
    ///
    /// # Remarks
    ///
    /// This method WILL NOT check if the file exists or not as this would
    /// require making use of blocking syscalls.
    ///
    pub fn get_manifest(&self, name: &str, reference: &str) -> PathBuf {
        self.manifests_dir
            .join(&name)
            .join(&reference)
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
    pub fn add_blob(&self, blob: &Path) -> Result<()> {
        let blob_digest = digest::retrieve_or_compute_and_store(blob)?;

        self.add_blob_with_digest(blob, &blob_digest)
    }

    pub fn add_blob_with_digest(&self, blob: &Path, digest: &str) -> Result<()> {
        let blob_filename = digest::prepend_sha_scheme(&digest);
        let blob_bucket_path = self.bucket_dir.join(blob_filename);

        std::fs::rename(
            blob,
            blob_bucket_path,
        )?;

        Ok(())
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
    pub fn add_manifest(&self, manifest: &Manifest) -> Result<String> {

        let manifest_json = serde_json::to_string_pretty(&manifest).unwrap();
        let manifest_json_digest = digest::compute_for_string(&manifest_json);
        let manifest_filename = digest::prepend_sha_scheme(&manifest_json_digest);
        let manifest_bucket_path = self.bucket_dir.join(&manifest_filename);

        let mut manifest_file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(&manifest_bucket_path)?;

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
    pub fn tag_manifest(&self, filename: &str, name: &str, reference: &str) -> Result<()> {
        let manifest_bucket_path = self.bucket_dir.join(filename);

        DirBuilder::new()
            .recursive(true)
            .create(self.manifests_dir.join(name))?;

        symlink(
            &manifest_bucket_path,
            self.manifests_dir
                .join(name)
                .join(reference),
        )?;

        Ok(())
    }

}

