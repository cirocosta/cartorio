extern crate serde;
extern crate serde_json;
extern crate tar;
extern crate tempfile;

use crate::digest;

use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use tempfile::tempdir;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RegistryDescriptor {
    media_type: &'static str,
    size: u64,
    digest: String,
}

/// A manifest that represents an image:
/// - configuration + layers.
///
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RegistryManifest {
    schema_version: u8,
    media_type: &'static str,
    config: RegistryDescriptor,
    layers: Vec<RegistryDescriptor>,
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
    /// Location of the container configuration
    config: String,
    repo_tags: Vec<String>,
    layers: Vec<String>,
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
/// └── manifests
///     └── library
///         └─ nginx
///           ├── latest -> ../../bucket/sha256:sha256(manifest_generated)
///           └── sha256:sha256(manifest_generated) --> ../../bucket/sha256:sha256(manifest_generated)
/// ```
///
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

    /// Creates the directories that represent the filesystem
    /// hierarch managed by BlobStore.
    ///
    fn create_directories(&self) -> std::io::Result<()> {
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
    /// TODO - move this to `DockerSavedImage`
    ///
    fn load_unpacked_tarball(&self, tarball_directory: &Path) {
        let tarball_manifest_json =
            std::fs::read_to_string(tarball_directory.join("manifest.json")).unwrap();

        let manifests = parse_docker_save_manifest(&tarball_manifest_json).unwrap();

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
    ///
    /// TODO - move this to `DockerSavedImage`
    /// TODO - split this into smaller functions
    ///
    fn move_tarball_content_to_bucket(
        &self,
        tarball_directory: &Path,
        manifest: &DockerSavedManifest,
    ) {
        // moving `config` over to `bucket`
        let config_tarball_path = tarball_directory.join(&manifest.config);
        let config_size = std::fs::metadata(&config_tarball_path).unwrap().len();
        let config_digest = digest::prepend_sha_scheme(&manifest.config_digest());

        let config_bucket_path = self.bucket_dir.join(&config_digest);

        assert!(std::fs::rename(config_tarball_path, config_bucket_path).is_ok());

        let config_descriptor = RegistryDescriptor {
            media_type: "application/vnd.docker.container.image.v1+json",
            size: config_size,
            digest: config_digest,
        };

        let mut layers_descriptors: Vec<RegistryDescriptor> =
            Vec::with_capacity(manifest.layers.len());

        // move each `layer` to the bucket
        for layer in &manifest.layers {
            let layer_tarball_path = tarball_directory.join(&layer);

            // compute the digest

            let layer_size = std::fs::metadata(&layer_tarball_path).unwrap().len();
            let layer_digest =
                digest::prepend_sha_scheme(&digest::compute_for_file(&layer_tarball_path).unwrap());
            let layer_bucket_path = self.bucket_dir.join(&layer_digest);

            assert!(std::fs::rename(layer_tarball_path, layer_bucket_path).is_ok());

            layers_descriptors.push(RegistryDescriptor {
                media_type: "application/vnd.docker.image.rootfs.diff.tar",
                size: layer_size,
                digest: layer_digest,
            });
        }

        // prepare the final manifest

        let registry_manifest = RegistryManifest {
            schema_version: 2,
            media_type: "application/vnd.docker.distribution.manifest.v2+json",
            config: config_descriptor,
            layers: layers_descriptors,
        };

        let registry_manifest_json = serde_json::to_string_pretty(&registry_manifest).unwrap();
        let registry_manifest_json_digest = digest::compute_for_string(&registry_manifest_json);
        let registry_manifest_bucket_path = self.bucket_dir.join(&registry_manifest_json_digest);

        let mut registry_manifest_file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(&registry_manifest_bucket_path)
            .unwrap();

        assert!(registry_manifest_file
            .write_all(registry_manifest_json.as_bytes())
            .is_ok());

        for repo_tag in &manifest.repo_tags {
            let mut repo_tag_splitted = repo_tag.split(':');

            let name = repo_tag_splitted.next().unwrap();
            let tag = repo_tag_splitted.next().unwrap();

            assert!(std::fs::DirBuilder::new()
                .recursive(true)
                .create(self.manifests_dir.join(&name))
                .is_ok());

            assert!(std::os::unix::fs::symlink(
                &registry_manifest_bucket_path,
                self.manifests_dir
                    .join(&name)
                    .join(&registry_manifest_json_digest),
            )
            .is_ok());

            assert!(std::os::unix::fs::symlink(
                &registry_manifest_bucket_path,
                self.manifests_dir.join(&name).join(&tag),
            )
            .is_ok());
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
