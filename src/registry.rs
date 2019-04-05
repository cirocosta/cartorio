extern crate serde;

use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestDescriptor {
    pub media_type: &'static str,
    pub size: u64,
    pub digest: String,
}


/// A manifest that represents an image:
/// - configuration + layers.
///
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    schema_version: u8,
    media_type: &'static str,
    config: ManifestDescriptor,
    layers: Vec<ManifestDescriptor>,
}
