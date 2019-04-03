extern crate tempfile;

use serde::{Deserialize};
use crate::blobstore::{BlobStore};
use crate::image_loader::{ImageLoader};
use std::io;
use tempfile::tempdir;
use std::path::PathBuf;


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
    pub fn new(tarball: PathBuf) -> io::Result<DockerSavedTarball> {
        let mut tarball_tmp_dir = tempdir().unwrap();
        let tarball_file = std::fs::File::open(tarball)?;

        tar::Archive::new(tarball_file)
            .unpack(tarball_tmp_dir.path())
            .unwrap();

        Ok(DockerSavedTarball{
            unpacked_dir: tarball_tmp_dir,
        })
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
        unimplemented!();
    }
}


/// Represents the configuration exposed by `docker save`d  tarballs.
///
/// ```text
/// {
///    Config: "$digest.json",
///    RepoTags: [ "name:tag" ],
///    Layers: [ "$digest/layer.tar" ]
/// }
/// ```
///
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct DockerSavedManifest {
    /// Location of the final container configuration.
    ///
    config: String,

    /// List of tags associated with the image.
    ///
    repo_tags: Vec<String>,

    /// Image layers.
    ///
    layers: Vec<String>,
}


impl DockerSavedManifest {

    /// Parses the contents of a manifest contained within a `docker save`d
    /// tarball.
    ///
    ///
    /// # Arguments
    ///
    /// * `content` - the contents of the `manifest.json` file.
    ///
    ///
    /// # Remarks
    ///
    /// The `manifest.json` file within a `docker save`d tarball may contain
    /// references to multiple images, thus, this method returns a vector instead
    /// of a single item.
    ///
    pub fn parse(content: &str) -> serde_json::Result<Vec<DockerSavedManifest>> {
        let manifests: Vec<DockerSavedManifest> = serde_json::from_str(content)?;
        Ok(manifests)
    }
}


#[cfg(test)]
mod docker_saved_manifest_tests {
    use super::*;

    #[test]
    fn test_parse_docker_save_manifest() {
        let data = r#"[
  {
    "Config": "48e2eeb489cdea15786d3622270750508d7385f3b684306703d17ffd50ecd34a.json",
    "RepoTags": [
      "a:latest"
    ],
    "Layers": [
      "4dc05cb02b54b373232011f781f8a98905d3e10575f2a399094f704d14913a7d/layer.tar"
    ]
  }
]"#;

        let manifests = parse_docker_save_manifest(data).unwrap();

        assert_eq!(manifests.len(), 1);
        assert_eq!(manifests[0].repo_tags.len(), 1,);
        assert_eq!(manifests[0].layers.len(), 1,);
        assert_eq!(
            manifests[0].config,
            "48e2eeb489cdea15786d3622270750508d7385f3b684306703d17ffd50ecd34a.json"
        );
    }
}
