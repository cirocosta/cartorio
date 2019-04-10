use cartorio::oci_image_layout::OciImageIndex;

const OCI_IMAGE_INDEX_SAMPLE: &'static str = r#"{
  "schemaVersion": 2,
  "manifests": [
    {
      "mediaType": "application/vnd.oci.image.manifest.v1+json",
      "digest": "sha256:dbaff2215b8b76ac78967a6602fdf80300a8de570f32cdbf7c21bffd49506d4b",
      "size": 347,
      "annotations": {
        "org.opencontainers.image.ref.name": "latest"
      },
      "platform": {
        "architecture": "amd64",
        "os": "linux"
      }
    }
  ]
}"#;

#[test]
fn parses_image_index() {
    let parsed: OciImageIndex = OCI_IMAGE_INDEX_SAMPLE.parse().unwrap();

}
