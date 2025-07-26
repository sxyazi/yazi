use std::path::Path;

use yazi_fs::services::Local;

#[inline]
pub async fn must_exists(path: impl AsRef<Path>) -> bool {
	Local::symlink_metadata(path).await.is_ok()
}

#[inline]
pub async fn maybe_exists(path: impl AsRef<Path>) -> bool {
	match Local::symlink_metadata(path).await {
		Ok(_) => true,
		Err(e) => e.kind() != std::io::ErrorKind::NotFound,
	}
}
