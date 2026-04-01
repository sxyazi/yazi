yazi_macro::mod_flat!(absolute metadata read_dir s3);

pub use absolute::try_absolute;
pub use read_dir::{DirEntry, ReadDir};
pub use s3::S3;
pub(crate) use s3::{copy_impl, copy_with_progress_impl};

type DynStore = std::sync::Arc<object_store::aws::AmazonS3>;

pub(super) fn init() {}
