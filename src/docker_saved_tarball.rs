use std::fs::File;
use std::fs;
use std::path::Path;

use tempfile::tempdir;

use crate::blobstore::{BlobStore};
use crate::digest;
use crate::docker_saved_manifest::{DockerSavedManifest, ImageManifest};
use crate::error::Result;
use crate::registry::{ManifestDescriptor, Manifest};


/// A tarball that has been generated through `docker save`.
///
pub struct DockerSavedTarball {

    /// Directory where the tarball has been unpacked
    /// (if so).
    ///
    unpacked_dir: tempfile::TempDir,

    /// The parsed verison of the `manifest.json` file that
    /// exists within a `docker save`d tarball containing the
    /// description of all of the images present in such tarball.
    ///
    parsed_manifest: DockerSavedManifest,

    /// The final owner of the blobs and manifests for the
    /// registry to serve.
    ///
    /// [`BlobStore]: struct.BlobStore.html
    ///
    blobstore: BlobStore,
}


impl DockerSavedTarball {

    /// Creates a new instance of DockerSavedTarball holding a reference
    /// to a temporary location to where the tarball gets extracted to.
    ///
    /// # Arguments
    ///
    /// * `tarball` - location of the tarball in the filesystem.
    ///
    ///
    /// # Remarks
    ///
    /// * This method *WILL* extract files from the tarball into the filesystem.
    /// * the temporary directory will be automatically removed once the object
    ///   goes out of scope.
    ///
    pub fn new(tarball: &Path, blobstore: BlobStore) -> Result<DockerSavedTarball> {
        let tarball_tmp_dir = tempdir().unwrap();
        let tarball_file = File::open(tarball)?;

        tar::Archive::new(tarball_file)
            .unpack(tarball_tmp_dir.path())?;

        let manifest_content = fs::read_to_string(tarball_tmp_dir
            .path()
            .join("manifest.json"))?;

        let parsed_manifest: DockerSavedManifest = manifest_content.parse()?;

        Ok(DockerSavedTarball{
            unpacked_dir: tarball_tmp_dir,
            parsed_manifest: parsed_manifest,
            blobstore: blobstore,
        })
    }

    /// Ingests a blob, computing the necessary metadata and moving the
    /// file to the blobstore.
    ///
    fn ingest_blob(&self, original_location: &Path, media_type: &'static str) -> Result<ManifestDescriptor> {
        let blob_digest = digest::compute_for_file(original_location)?;

        digest::store(original_location, &blob_digest)?;

        let blob_metadata = std::fs::metadata(original_location)?;
        let blob_size = blob_metadata.len();

        self.blobstore.add_blob(original_location)?;

        Ok(ManifestDescriptor{
            media_type: media_type,
            size: blob_size,
            digest: blob_digest,
        })
    }

    fn ingest_config(&self, original_location: &Path) -> Result<ManifestDescriptor> {
        self.ingest_blob(original_location, "application/vnd.docker.container.image.v1+json")
    }

    fn ingest_layer(&self, original_location: &Path) -> Result<ManifestDescriptor> {
        self.ingest_blob(original_location, "application/vnd.docker.image.rootfs.diff.tar")
    }

    /// Loads a single image as described by a manifest.
    ///
    fn load_image(&self, manifest: &ImageManifest) -> Result<()> {
        let config_descriptor = self.ingest_config(
            &self.unpacked_dir.path().join(&manifest.config),
        )?;

        let mut layers_descriptors: Vec<ManifestDescriptor> = 
            Vec::with_capacity(manifest.layers.len() + 1);

        for layer in &manifest.layers {
            layers_descriptors.push(self.ingest_layer(
                &self.unpacked_dir.path().join(&layer),
            )?);
        }


        let manifest_filename = self.
            ingest_manifest(config_descriptor, layers_descriptors)?;


        for repo_tag in &manifest.repo_tags {
            let mut repo_tag_splitted = repo_tag.split(':');

            let name = repo_tag_splitted.next().unwrap();
            let tag = repo_tag_splitted.next().unwrap();

            self.blobstore.tag_manifest(&manifest_filename, &name, &tag)?;
            self.blobstore.tag_manifest(&manifest_filename, &name, &manifest_filename)?;
        }

        Ok(())
    }


    /// todo
    ///
    /// ```txt
    ///
    ///   create manifest from `config` + `layer_descriptors`
    ///   write to file in the right place
    ///
    /// ```
    ///
    fn ingest_manifest(&self, 
        config_desc: ManifestDescriptor, layers_descs: Vec<ManifestDescriptor>,
    ) -> Result<String> {

        let manifest = Manifest {
            schema_version: 2,
            media_type: "application/vnd.docker.distribution.manifest.v2+json",
            config: config_desc,
            layers: layers_descs,
        };

        let manifest_filename = self.blobstore.add_manifest(&manifest)?;

        Ok(manifest_filename)
    }

    /// Loads the contents of all of the images in the contents of a `docker save`d 
    /// tarball into the blobstore.
    ///
    /// ```txt
    /// FOR EACH IMAGE IN MANIFEST:
    ///
    ///    CONFIG
    ///     descriptors = append(descriptors, ingest_blob())
    ///    
    ///       1. compute the digest
    ///       2. gather the size
    ///       4. move file to blobstore
    ///       3. create descriptor
    ///    
    ///    
    ///    FOR EACH LAYER
    ///    
    ///     descriptors = append(descriptors, ingest_blob())
    ///       1. compute the digest
    ///       2. gather the size on disk
    ///       4. move file to blobstore
    ///       3. create descriptor
    ///    
    ///    
    ///    MANIFEST
    ///     1. create manifest using `config` + `layer_descriptors`
    ///     2. write to file
    ///     3. compute digest
    ///     4. move to blobstore
    ///     5. link tags to manifest
    /// ```
    ///
    /// # Arguments
    ///
    /// * `blobstore` - a [`Blobstore`] that represents the destination of
    /// contents of this tarball.
    ///
    ///
    /// [`BlobStore`]: struct.BlobStore.html
    ///
    pub fn load(&self) -> Result<()> {

        for image_manifest in &self.parsed_manifest.images_manifests {
            self.load_image(&image_manifest)?;
        }

        Ok(())
    }
}


