extern crate serde_json;

use serde::Deserialize;
use std::str::FromStr;
use std::path::Path;


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
pub struct ImageManifest {
    /// Location of the final container configuration.
    ///
    pub config: String,

    /// List of tags associated with the image.
    ///
    pub repo_tags: Vec<String>,

    /// Image layers.
    ///
    pub layers: Vec<String>,
}

pub struct DockerSavedManifest {
    pub images_manifests: Vec<ImageManifest>,
}

impl FromStr for DockerSavedManifest {
    
    type Err = serde_json::Error;

    /// Parses the contents of a multi-image manifest contained within a 
    /// `docker save`d tarball.
    ///
    ///
    /// # Arguments
    ///
    /// * `content` - the contents of the `manifest.json` file.
    ///
    fn from_str(content: &str) -> Result<Self, Self::Err> {
        let images_manifests: Vec<ImageManifest> = serde_json::from_str(content)?;

        Ok(DockerSavedManifest{
            images_manifests: images_manifests,
        })
    }

}


#[cfg(test)]
mod docker_saved_manifest_tests {
    use super::*;

    #[test]
    fn test_docker_saved_manifest_from_str() {
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

        let manifests: DockerSavedManifest = data.parse().unwrap();

        assert_eq!(manifests.images_manifests.len(), 1);
        assert_eq!(manifests.images_manifests[0].repo_tags.len(), 1,);
        assert_eq!(manifests.images_manifests[0].layers.len(), 1,);
        assert_eq!(
            manifests.images_manifests[0].config,
            "48e2eeb489cdea15786d3622270750508d7385f3b684306703d17ffd50ecd34a.json"
        );
    }
}
