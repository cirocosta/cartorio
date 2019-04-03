use tempfile::tempdir;
use cartorio::blobstore::BlobStore;
use std::fs;

#[test]
fn test_blobstore_new() {
    let mut root_dir = tempdir().unwrap();

    let blobstore = BlobStore::new(root_dir.path()).unwrap();
    
    assert!(fs::metadata(blobstore.bucket_dir).unwrap().is_dir());
    assert!(fs::metadata(blobstore.manifests_dir).unwrap().is_dir());
}
