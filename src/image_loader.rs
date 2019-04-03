use std::io;
use crate::blobstore::{BlobStore};

/// Something that can load images into a blobstore.
///
pub trait ImageLoader {

    /// Loads images into the blobstore.
    ///
    fn load(&self, blobstore: &BlobStore) -> io::Result<()>;
}
