use tempfile::tempdir;
use std::path::{PathBuf};
use cartorio::image_loader::ImageLoader;
use cartorio::docker_saved_tarball::DockerSavedTarball;
use cartorio::blobstore::BlobStore;

#[test]
fn test_docker_saved_tarball() {
    let repository_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let tarball_path = repository_root.join("tests/fixtures/small-image/image.tar");
    let docker_saved_tarball = DockerSavedTarball::new(&tarball_path).unwrap();

    let blobstore_root_dir = tempdir().unwrap();
    let blobstore = BlobStore::new(blobstore_root_dir.path()).unwrap();

    assert!(docker_saved_tarball.load(&blobstore).is_ok());
}
