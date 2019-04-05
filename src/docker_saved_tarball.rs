extern crate tempfile;

use crate::blobstore::{BlobStore};
use crate::docker_saved_manifest::DockerSavedManifest;
use crate::image_loader::{ImageLoader};
use crate::registry::{ManifestDescriptor};
use tempfile::tempdir;
use std::fs::File;
use std::fs;
use std::io::{Read};
use std::io;
use std::path::{PathBuf, Path};


/// A tarball that has been generated through `docker save`.
///
pub struct DockerSavedTarball {

    /// Directory where the tarball has been unpacked
    /// (if so).
    ///
    unpacked_dir: tempfile::TempDir,
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
    pub fn new(tarball: &Path) -> io::Result<DockerSavedTarball> {
        let mut tarball_tmp_dir = tempdir().unwrap();
        let tarball_file = File::open(tarball)?;

        tar::Archive::new(tarball_file)
            .unpack(tarball_tmp_dir.path())
            .unwrap();

        let manifest_content = fs::read_to_string(tarball)?;

        Ok(DockerSavedTarball{
            unpacked_dir: tarball_tmp_dir,
        })
    }


    fn prepare_blob(original_location: &Path) -> io::Result<ManifestDescriptor> {
        unimplemented!();
    }
}


impl ImageLoader for DockerSavedTarball {

    /// Loads the contents of the `docker save`d tarball into the
    /// blobstore.
    ///
    ///
    /// # Arguments
    ///
    /// * `blobstore` - a [`Blobstore`] that represents the destination of
    /// contents of this tarball.
    ///
    ///
    /// [`BlobStore`]: struct.BlobStore.html
    ///
    fn load(&self, blobstore: &BlobStore) -> io::Result<()> {

        // CONFIG
        //  1. compute the digest
        //  2. gather the size
        //  4. move file to blobstore
        //  3. create descriptor

        // FOR EACH LAYER
        //  1. compute the digest
        //  2. gather the size on disk
        //  4. move file to blobstore
        //  3. create descriptor
        
        // MANIFEST
        //  1. create manifest using `config` + `layer_descriptors`
        //  2. write to file
        //  3. compute digest
        //  4. move to blobstore
        //  5. link tags to manifest
        
        unimplemented!();
    }
}


