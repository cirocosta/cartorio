extern crate serde;

use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ManifestDescriptor {
    media_type: &'static str,
    size: u64,
    digest: String,
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
