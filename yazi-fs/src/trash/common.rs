#[cfg(trash_unix)]
pub(super) fn restore_item(from: &std::path::Path, to: &std::path::Path) -> std::io::Result<()> {
	use std::{fs, io};

	let is_dir = fs::symlink_metadata(from)?.is_dir();
	if let Some(parent) = to.parent() {
		fs::create_dir_all(parent)?;
	}

	match if is_dir { fs::create_dir(to) } else { fs::File::create_new(to).map(|_| ()) } {
		Ok(()) => fs::rename(from, to),
		Err(e) if e.kind() == io::ErrorKind::AlreadyExists => Err(io::Error::new(
			io::ErrorKind::AlreadyExists,
			format!("restore target already exists: {to:?}"),
		)),
		Err(e) => Err(e),
	}
}
