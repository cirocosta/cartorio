use crate::blobstore::{BlobStore};

/// Something that can load images into a blobstore.
///
trait ImageLoader {

    /// Loads images into the blobstore.
    ///
    fn load(&self, blobstore: &BlobStore) -> io::Result<()>;
}
