use std::path::{Path, PathBuf};
use std::fs;

use crate::concourse_resource_metadata::{ConcourseResourceMetadata};
use crate::blobstore::{BlobStore};
use crate::error::Result;


pub struct ConcourseImageResource {

    /// Root directory of the image resource.
    ///
    root_dir: PathBuf,

    /// Path to the rootfs.tgz file.
    ///
    rootfs_path: PathBuf,

    /// The parsed `resource_metadata.json` file that represents the metadata regarding the
    /// resource under `root_dir`.
    ///
    resource_metadata: ConcourseResourceMetadata,
}

impl ConcourseImageResource {

    /// Instantiates a new ConcourseImageResource after validating that the Concourse
    /// `image_resource` interface is satisfied for such directory.
    ///
    /// # Arguments
    ///
    /// * `dir` - location where the contents live.
    /// * `blobstore` - a BlobStore to own the resulting image from such directory.
    ///
    pub fn new(dir: &Path, blobstore: BlobStore) -> Result<ConcourseImageResource> {
        let metadata_content = fs::read_to_string(dir.join("resource_metadata.json"))?;
        let metadata: ConcourseResourceMetadata = metadata_content.parse()?;

        let rootfs_tgz = dir.join("rootfs.tgz");
        if !rootfs_tgz.exists() {
            failure::format_err!("no rootfs.tgz in dir %s");
        }

        Ok(ConcourseImageResource{
            root_dir: dir.to_owned(),
            resource_metadata: metadata,
            rootfs_path: rootfs_tgz,
        })
    }

    /// Loads the contents found in the resource image directory into the blobstore.
    ///
    /// ```txt
    ///
    ///    dir
    ///    ├── resource_metadata.json
    ///    └── rootfs.tgz
    ///
    /// ==>
    ///
    /// 1. generate layer image configuration
    /// 2. compute digest of the `rootfs.tgz` layer
    /// 3. generate manifest pointing to the layer and the config
    ///
    /// ==>
    ///
    ///    .
    ///    ├── bucket
    ///    │   ├── sha256:aaabbbccc...  (rootfs.tgz)
    ///    │   ├── sha256:dddeeefff...  (rootfs.tgz' config)
    ///    │   └── sha256:ggghhhiii...  (manifest)
    ///    │
    ///    └── manifests
    ///        └── library
    ///            └─ $name (from `resource_metadata.json`)k
    ///              ├── latest -> ../../bucket/sha256:ggghhhiii...
    ///              ├── $version (from `resource_metadata.json`) -> ../../bucket/sha256:ggghhhiii...
    ///              └── sha256:dbafff2... --> ../../bucket/ggghhhiii...
    ///
    /// ```
    ///
    pub fn load(&self) -> Result<()> {
        Ok(())
    }
}
