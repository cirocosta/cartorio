use std::str::FromStr;

use serde::Deserialize;

/// Metadata as supplied by the implementor of the Concourse image_resource
/// interface.
///
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConcourseResourceMetadata {
    #[serde(rename = "type")]
    imageType: String,
    version: String,
}

impl FromStr for ConcourseResourceMetadata {
    type Err = serde_json::Error;

    fn from_str(content: &str) -> Result<Self, Self::Err> {
        let metadata: ConcourseResourceMetadata = serde_json::from_str(content)?;

        Ok(metadata)
    }
}

#[cfg(test)]
mod concourse_resource_metadata_tests {
    use super::*;

    const RESOURCE_METADATA_SAMPLE: &'static str = r#"{
  "type": "registry-image",
  "version": "v1.2.3"
}"#;

    #[test]
    fn parses_metadata_json() {
        let parsed: ConcourseResourceMetadata = RESOURCE_METADATA_SAMPLE.parse().unwrap();
    }
}
