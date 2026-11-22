use std::{fs::canonicalize, io, path::Path};

use yazi_shim::path::OsPathExt;

use crate::casefold::Casefold;

pub struct Identity;

impl Identity {
	/// Compares the final entries without following a symlink at the end of either path.
	///
	/// - If `b` is a symlink to directory `a`, `a/foo` and `b/foo` are the same entry,
	///   even if `foo` itself is a dangling symlink.
	/// - If `a/alias` is a symlink to `a/foo`, `a/alias` and `a/foo` are different entries.
	/// - If `a/bar` is a hard link to `a/foo`, they are different entries on Unix and Windows.
	pub async fn same_entry<P, Q>(a: P, b: Q) -> io::Result<bool>
	where
		P: AsRef<Path>,
		Q: AsRef<Path>,
	{
		let (a, b) = (a.as_ref().to_owned(), b.as_ref().to_owned());

		tokio::task::spawn_blocking(move || Self::same_entry_impl(&a, &b)).await?
	}

	pub async fn same_file(a: &Path, b: &Path) -> io::Result<bool> {
		let (a, b) = (a.to_owned(), b.to_owned());

		tokio::task::spawn_blocking(move || Self::same_file_impl(&a, &b)).await?
	}

	fn same_parent(a: &Path, b: &Path) -> io::Result<bool> {
		let a = a.non_empty().unwrap_or(Path::new("."));
		let b = b.non_empty().unwrap_or(Path::new("."));

		Self::same_file_impl(a, b)
	}

	pub(super) fn same_entry_path(a: &Path, b: &Path) -> io::Result<bool> {
		match (a.pair(), b.pair()) {
			(Some((a_p, a_n)), Some((b_p, b_n))) => Ok(
				Self::same_parent(a_p, b_p)?
					&& (a_n == b_n || Casefold::final_name(a)? == Casefold::final_name(b)?),
			),
			// Neither `.` nor `/` has a filename. After `cd /`, they refer to the same directory.
			(None, None) => Ok(canonicalize(a)? == canonicalize(b)?),
			_ => Ok(false),
		}
	}
}
