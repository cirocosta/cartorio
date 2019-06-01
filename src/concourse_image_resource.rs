use std::fs;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

use flate2::read::GzDecoder;

use crate::blobstore::BlobStore;
use crate::concourse_resource_metadata::ConcourseResourceMetadata;
use crate::digest;
use crate::error::Result;
use crate::image_config::ImageConfig;
use crate::registry::{ManifestDescriptor, Manifest};

pub struct ConcourseImageResource {
    /// Root directory of the image resource.
    ///
    root_dir: PathBuf,

    /// Path to the rootfs.tgz file.
    ///
    rootfs_path: PathBuf,

    /// The parsed `resource_metadata.json` file that represents the metadata regarding the
    /// resource under `root_dir`.
    ///
    resource_metadata: ConcourseResourceMetadata,

    /// The final owner of the blobs and manifests for the
    /// registry to serve.
    ///
    /// [`BlobStore]: struct.BlobStore.html
    ///
    blobstore: BlobStore,
}

impl ConcourseImageResource {
    /// Instantiates a new ConcourseImageResource after validating that the Concourse
    /// `image_resource` interface is satisfied for such directory.
    ///
    /// # Arguments
    ///
    /// * `dir` - location where the contents live.
    /// * `blobstore` - a BlobStore to own the resulting image from such directory.
    ///
    pub fn new(dir: &Path, blobstore: BlobStore) -> Result<ConcourseImageResource> {
        let metadata_content = fs::read_to_string(dir.join("resource_metadata.json"))?;

        let metadata: ConcourseResourceMetadata = metadata_content.parse()?;
        let rootfs_tgz = dir.join("rootfs.tgz");

        if !rootfs_tgz.exists() {
            return Err(failure::format_err!("no rootfs.tgz in dir %s"));
        }

        Ok(ConcourseImageResource {
            blobstore: blobstore,
            resource_metadata: metadata,
            root_dir: dir.to_owned(),
            rootfs_path: rootfs_tgz,
        })
    }

    /// Loads the contents found in the resource image directory into the blobstore.
    ///
    /// ```txt
    ///
    ///    dir
    ///    ├── resource_metadata.json
    ///    └── rootfs.tgz
    ///
    /// ==>
    ///
    /// 1. generate layer image configuration
    ///     - diff_ids must be generated too
    ///         --> ingest the layer
    /// 2. ingest the layer
    /// 3. generate manifest pointing to the layer and the config
    ///
    /// ==>
    ///
    ///    .
    ///    ├── bucket
    ///    │   ├── sha256:aaabbbccc...  (rootfs.tgz)
    ///    │   ├── sha256:dddeeefff...  (rootfs.tgz' config)
    ///    │   └── sha256:ggghhhiii...  (manifest)
    ///    │
    ///    └── manifests
    ///        └── library
    ///            └─ $name (from `resource_metadata.json`)k
    ///              ├── latest -> ../../bucket/sha256:ggghhhiii...
    ///              ├── $version (from `resource_metadata.json`) -> ../../bucket/sha256:ggghhhiii...
    ///              └── sha256:dbafff2... --> ../../bucket/ggghhhiii...
    ///
    /// ```
    ///
    pub fn load(&self) -> Result<()> {
        self.decompress_rootfs()?;

        let layer_descriptor = self.ingest_rootfs()?;
        let image_config = self.generate_config(&layer_descriptor.digest)?;
        let config_descriptor = self.ingest_config(&self.root_dir.join("config.json"))?;

        let manifest = Manifest {
            schema_version: 2,
            media_type: "application/vnd.docker.distribution.manifest.v2+json",
            config: config_descriptor,
            layers: vec![layer_descriptor],
        };

        let manifest_filename = self.blobstore.add_manifest(&manifest)?;

        self.blobstore.tag_manifest(
            &manifest_filename, 
            &self.resource_metadata.image_type, 
            &manifest_filename,
        );

        self.blobstore.tag_manifest(
            &manifest_filename, 
            &self.resource_metadata.image_type, 
            &self.resource_metadata.version,
        );

        Ok(())
    }

    /// Ingests a blob, computing the necessary metadata and moving the
    /// file to the blobstore.
    ///
    fn ingest_blob(
        &self,
        original_location: &Path,
        media_type: &'static str,
    ) -> Result<ManifestDescriptor> {
        let blob_digest = digest::compute_for_file_and_store(original_location)?;
        let blob_metadata = std::fs::metadata(original_location)?;
        let blob_size = blob_metadata.len();

        self.blobstore.add_blob(original_location)?;

        Ok(ManifestDescriptor {
            media_type: media_type,
            size: blob_size,
            digest: digest::prepend_sha_scheme(&blob_digest),
        })
    }

    fn decompress_rootfs(&self) -> Result<()> {
        let tar_gz = fs::File::open(&self.rootfs_path)?;
        let mut tar = GzDecoder::new(tar_gz);
        let mut tar_file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(&self.root_dir.join("rootfs.tar"))?;

        let _result = std::io::copy(&mut tar, &mut tar_file)?;

        Ok(())
    }

    fn generate_config(&self, layer_digest: &str) -> Result<ImageConfig> {
        let config = ImageConfig::new(vec![layer_digest.to_owned()]);

        let mut config_file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(&self.root_dir.join("config.json"))?;

        config_file.write_all(config.to_string().as_bytes())?;

        Ok(config)
    }

    fn ingest_config(&self, original_location: &Path) -> Result<ManifestDescriptor> {
        self.ingest_blob(
            original_location,
            "application/vnd.docker.container.image.v1+json",
        )
    }

    fn ingest_rootfs(&self) -> Result<ManifestDescriptor> {
        self.ingest_blob(
            &self.root_dir.join("rootfs.tar"),
            "application/vnd.docker.image.rootfs.diff.tar",
        )
    }
}
