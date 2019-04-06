use tempfile::tempdir;
use cartorio::blobstore::BlobStore;
use std::fs;

#[test]
fn test_blobstore_new() {
    let root_dir = tempdir().unwrap();
    let blobstore = BlobStore::new(root_dir.path()).unwrap();
    
    assert!(fs::metadata(blobstore.bucket_dir).unwrap().is_dir());
    assert!(fs::metadata(blobstore.manifests_dir).unwrap().is_dir());
}


#[test]
fn test_blobstore_add_blob() {
    let root_dir = tempdir().unwrap();
    let blobstore = BlobStore::new(root_dir.path()).unwrap();
    let blob_path = root_dir.path().join("file.txt");

    fs::write(&blob_path, "something")
        .expect("writes to file");

    blobstore.add_blob(&blob_path)
        .expect("adds blob");

    let mut bucket_dir_entries = fs::read_dir(&blobstore.bucket_dir).unwrap();
    let first_entry = bucket_dir_entries.next().unwrap();
    let filepath = first_entry.unwrap().path();

    assert_eq!(
        filepath.file_name().unwrap(), 
        "sha256:3fc9b689459d738f8c88a3a48aa9e33542016b7a4052e001aaa536fca74813cb",
    );

    assert!(
        bucket_dir_entries.next().is_none(),
    );
}
