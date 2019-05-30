use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// References the layer content addresses used by the image, making the image config hash depend
/// on the filesystem hash.
///
#[derive(Serialize, Deserialize)]
pub struct ImageConfigRootfs {
    /// Must be set to `layers`.
    ///
    #[serde(rename = "type")]
    pub rootfsType: String,

    /// An array of layer content hashes in order from first to last.
    ///
    /// A DiffID corresponds to the digest over the layer's UNCOMPRESSED tar archive and serialized
    /// in the descriptor digest format - not to be confused with `layer digests` (referenced in
    /// the manifest).
    ///
    pub diff_ids: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct ImageConfig {
    /// The CPU architecture which the binaries in this image are built to run on.
    ///
    /// Values provided here should be according to [`GOARCH`][goarch].
    ///
    /// [goarch]: https://golang.org/doc/install/source#environment
    ///
    pub architecture: String,

    /// The name of the operating system which the image is built to run on.
    ///
    /// Values provided here should be according to [`GOOS`][goos].
    ///
    /// [goos]: https://golang.org/doc/install/source#environment
    ///
    pub os: String,

    pub rootfs: ImageConfigRootfs,
}

impl ImageConfig {
    /// Instantiates a new ImageConfig using a default configuration that lacks just diff ids in
    /// the rootfs.
    ///
    pub fn new(diff_ids: Vec<String>) -> ImageConfig {
        ImageConfig {
            architecture: "amd64".to_owned(),
            os: "linux".to_owned(),
            rootfs: ImageConfigRootfs {
                rootfsType: "layers".to_owned(),
                diff_ids: diff_ids,
            },
        }
    }
}

impl FromStr for ImageConfig {
    type Err = serde_json::Error;

    fn from_str(content: &str) -> Result<Self, Self::Err> {
        let config: ImageConfig = serde_json::from_str(content)?;

        Ok(config)
    }
}

impl fmt::Display for ImageConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let config = match serde_json::to_string_pretty(self) {
            Err(_) => return Err(fmt::Error),
            Ok(c) => c,
        };

        write!(f, "{}", config)
    }
}

#[cfg(test)]
mod image_config_tests {
    use super::*;

    const IMAGE_CONFIG_SAMPLE: &'static str = r#"{
  "architecture": "amd64",
  "os": "linux",
  "rootfs": {
    "type": "layers",
    "diff_ids": [
      "id1"
    ]
  }
}"#;

    #[test]
    fn unmarshal() {
        let parsed: ImageConfig = IMAGE_CONFIG_SAMPLE.parse().unwrap();

        assert_eq!(parsed.architecture, "amd64");
        assert_eq!(parsed.rootfs.diff_ids[0], "id1");
    }

    #[test]
    fn marshal() {
        let mut diff_ids: Vec<String> = Vec::new();
        diff_ids.push("id1".to_owned());

        let configuration = ImageConfig::new(diff_ids);

        assert_eq!(configuration.to_string(), IMAGE_CONFIG_SAMPLE);
    }
}
