use std::fs;
use std::io::prelude::*;
use std::path::PathBuf;

use tempfile::tempdir;

use cartorio::blobstore::BlobStore;
use cartorio::concourse_image_resource::ConcourseImageResource;

const SAMPLE_RESOURCE_METADATA: &'static str = r#"{
  "type": "registry-image",
  "version": "v1.2.3"
}"#;

mod load {
    use super::*;

    #[test]
    fn writes_layer_configuration_to_bucket() {
        let blobstore_root_dir = tempdir().unwrap();
        let blobstore = BlobStore::new(blobstore_root_dir.path()).unwrap();

        let repository_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let original_resource_dir = repository_root.join("tests/fixtures/resource");

        let resource_dir = tempdir().unwrap();

        fs::copy(
            original_resource_dir.join("resource_metadata.json"),
            resource_dir.path().join("resource_metadata.json"),
        )
        .unwrap();

        fs::copy(
            original_resource_dir.join("rootfs.tgz"),
            resource_dir.path().join("rootfs.tgz"),
        )
        .unwrap();

        let loader = ConcourseImageResource::new(resource_dir.path(), blobstore).unwrap();
        assert!(loader.load().is_ok());

        // TODO read the config file
    }

}

mod new {
    use super::*;

    #[test]
    fn fails_without_resource_metadata_in_dir() {
        let blobstore_root_dir = tempdir().unwrap();
        let blobstore = BlobStore::new(blobstore_root_dir.path()).unwrap();

        let resource_dir = tempdir().unwrap();

        assert!(ConcourseImageResource::new(resource_dir.path(), blobstore,).is_err(),);
    }

    #[test]
    fn fails_with_unparseable_resource_metadata() {
        let blobstore_root_dir = tempdir().unwrap();
        let blobstore = BlobStore::new(blobstore_root_dir.path()).unwrap();

        let resource_dir = tempdir().unwrap();

        let mut metadata_file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(&resource_dir.path().join("resource_metadata.json"))
            .unwrap();

        metadata_file.write_all(b"ahuah").unwrap();

        assert!(ConcourseImageResource::new(resource_dir.path(), blobstore,).is_err());
    }

    #[test]
    fn fails_without_rootfs() {
        let blobstore_root_dir = tempdir().unwrap();
        let blobstore = BlobStore::new(blobstore_root_dir.path()).unwrap();

        let repository_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let original_resource_dir = repository_root.join("tests/fixtures/resource");

        let resource_dir = tempdir().unwrap();

        fs::copy(
            original_resource_dir.join("resource_metadata.json"),
            resource_dir.path().join("resource_metadata.json"),
        )
        .unwrap();

        assert!(ConcourseImageResource::new(resource_dir.path(), blobstore).is_err());
    }
}
