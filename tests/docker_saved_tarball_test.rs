use std::path::{PathBuf};
use cartorio::docker_saved_tarball::{DockerSavedTarball};

#[test]
fn test_docker_saved_tarball() {
    let repository_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let tarball_path = repository_root.join("tests/fixtures/small-image/image.tar");
    let docker_saved_tarball = DockerSavedTarball::new(tarball_path.to_path_buf()).unwrap();
}
