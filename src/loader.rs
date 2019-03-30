extern crate serde;
extern crate serde_json;
extern crate tar;
extern crate tempfile;

use serde::Deserialize;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

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
    /// Location of the container configuration
    config: String,
    repo_tags: Vec<String>,
    layers: Vec<String>,
}

fn prepend_sha_scheme(digest: &str) -> String {
    let mut res = String::with_capacity(2);
    res.push_str("sha256:");
    res.push_str(digest);
    res.to_string()
}

impl DockerSavedManifest {
    /// Extracts the `digest` part of the configuration.
    ///
    /// ```txt
    /// digest.json --> digest
    /// ```
    ///
    fn config_digest(&self) -> &str {
        self.config.split('.').next().unwrap()
    }

    /// Extracts the `digest` part of a layer.
    ///
    /// ```txt
    /// digest/layer.tar --> digest
    /// ```
    ///
    fn layer_digest(layer: &str) -> &str {
        layer.split('/').next().unwrap()
    }
}

fn parse_docker_save_manifest(content: &str) -> serde_json::Result<Vec<DockerSavedManifest>> {
    let manifests: Vec<DockerSavedManifest> = serde_json::from_str(content)?;
    Ok(manifests)
}

#[cfg(test)]
mod docker_saved_manifest_tests {
    use super::*;

    #[test]
    fn test_digest_retrieval() {
        let manifest = DockerSavedManifest {
            config: "abcdef.json".to_string(),
            layers: vec!["0123/layer.tar".to_string()],
            repo_tags: vec!["name:tag".to_string()],
        };

        assert_eq!(manifest.config_digest(), "abcdef");
        assert_eq!(
            DockerSavedManifest::layer_digest(&manifest.layers[0]),
            "0123"
        );
    }

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

/// Manages the location where all of the files managed by
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
/// ├── blobs
/// │   └── library
/// │       └─ nginx
/// │	    ├── sha256:48e2eeb489cde....03d17ffd50ecd34a -> ../../bucket/sha256:48e2eeb48.... (config)
/// │	    └── sha256:4dc05cb02b54b....04d14913a7d -> ../../bucket/sha256:4dc0..	      (layer)
/// │
/// └── manifests
///     └── library
///         └─ nginx
///           ├── latest -> ../../bucket/sha256:sha256(manifest_generated)
///           └── sha256:sha256(manifest_generated) --> ../../bucket/sha256:sha256(manifest_generated)
/// ```
///
struct BlobStore {
    bucket_dir: PathBuf,
    blobs_dir: PathBuf,
    manifests_dir: PathBuf,
}

impl BlobStore {
    fn new(root: &str) -> BlobStore {
        BlobStore {
            bucket_dir: Path::new(root).join("bucket"),
            blobs_dir: Path::new(root).join("blobs"),
            manifests_dir: Path::new(root).join("manifests"),
        }
    }

    /// Creates the directories that represent the filesystem
    /// hierarch managed by BlobStore.
    ///
    fn create_directories(&self) -> std::io::Result<()> {
        std::fs::DirBuilder::new()
            .recursive(true)
            .create(&self.blobs_dir)?;

        std::fs::DirBuilder::new()
            .recursive(true)
            .create(&self.bucket_dir)?;

        std::fs::DirBuilder::new()
            .recursive(true)
            .create(&self.manifests_dir)
    }

    /// Loads the contents of a tarball that has been extracted into
    /// a directory.
    ///
    /// # Arguments
    ///
    /// * `directory` - the directory where the extracted contents of the
    /// tarball lives.
    ///
    ///
    fn load_unpacked_tarball(&self, tarball_directory: &Path) {
        let manifests =
            parse_docker_save_manifest(tarball_directory.join("manifest.json").to_str().unwrap())
                .unwrap();

        for manifest in manifests {
            self.move_tarball_content_to_bucket(&tarball_directory, &manifest);
        }
    }

    /// Moves the contents that are referenced in the manifest over
    /// to the blobs directory.
    ///
    /// # Arguments
    ///
    /// * `manifest` - the manifest that describes the contents to be moved.
    /// * `tarball_directory` - directory where the docker tarball contents
    ///                         were extracted to.
    ///
    fn move_tarball_content_to_bucket(
        &self,
        tarball_directory: &Path,
        manifest: &DockerSavedManifest,
    ) {
        // moving `config` over to `bucket`
        let config_tarball_path = tarball_directory.join(&manifest.config);
        let config_bucket_path = self
            .bucket_dir
            .join(prepend_sha_scheme(&manifest.config_digest()));

        std::fs::rename(config_tarball_path, config_bucket_path);

        // move each `layer` to the bucket
        for layer in &manifest.layers {
            let layer_tarball_path = tarball_directory.join(&layer).join("layer.tar");
            let layer_bucket_path =
                self.bucket_dir
                    .join(prepend_sha_scheme(DockerSavedManifest::layer_digest(
                        &layer,
                    )));

            std::fs::rename(layer_tarball_path, layer_bucket_path);
        }
    }
}

/// Loads tarballs from `docker save` into the cartorio's
/// filesystem hierarchy created at root directory.
///
/// # Arguments
///
/// * `blobstore_dir` - directory where the registry file hierarchy gets created.
/// * `tarballs` - list of tarballs to load.
///
pub fn load_tarball(blobstore_dir: &str, tarball: &str) {
    let blobstore = BlobStore::new(blobstore_dir);

    assert!(blobstore.create_directories().is_ok());

    let tarball_tmp_dir = tempdir().unwrap();
    let tarball_file = std::fs::File::open(tarball).unwrap();

    tar::Archive::new(tarball_file)
        .unpack(tarball_tmp_dir.path())
        .unwrap();

    blobstore.load_unpacked_tarball(tarball_tmp_dir.path());
}
