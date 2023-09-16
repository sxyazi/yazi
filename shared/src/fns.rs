use std::{borrow::Cow, env, path::{Component, Path, PathBuf}};

use tokio::fs;

use crate::Url;

pub fn expand_path(p: impl AsRef<Path>) -> PathBuf {
	let p = p.as_ref();
	if let Ok(p) = p.strip_prefix("~") {
		if let Some(home) = env::var_os("HOME") {
			return PathBuf::from_iter([&home, p.as_os_str()]);
		}
	}
	if p.is_absolute() {
		return p.to_path_buf();
	}
	env::current_dir().map_or_else(|_| p.to_path_buf(), |c| c.join(p))
}

#[inline]
pub fn expand_url(mut u: Url) -> Url {
	u.set_path(expand_path(&u));
	u
}

pub fn short_path(p: &Path, base: &Path) -> String {
	if let Ok(p) = p.strip_prefix(base) {
		return p.display().to_string();
	}
	p.display().to_string()
}

pub fn readable_path(p: &Path) -> String {
	if let Ok(home) = env::var("HOME") {
		if let Ok(p) = p.strip_prefix(home) {
			return format!("~/{}", p.display());
		}
	}
	p.display().to_string()
}

pub fn readable_size(size: u64) -> String {
	let units = ["B", "KB", "MB", "GB", "TB", "PB", "EB"];
	let mut size = size as f64;
	let mut i = 0;
	while size > 1024.0 && i < units.len() - 1 {
		size /= 1024.0;
		i += 1;
	}
	format!("{:.1} {}", size, units[i])
}

pub async fn unique_path(mut p: Url) -> Url {
	let Some(name) = p.file_name().map(|n| n.to_owned()) else {
		return p;
	};

	let mut i = 0;
	while fs::symlink_metadata(&p).await.is_ok() {
		i += 1;
		let mut name = name.clone();
		name.push(format!("_{i}"));
		p.set_file_name(name);
	}
	p
}

// Parmaters
// * `path`: The absolute path(contains no `/./`) to get relative path.
// * `root`: The absolute path(contains no `/./`) to be compared.
//
// Return
// * Unix: The relative format to `root` of `path`.
// * Windows: The relative format to `root` of `path`; or `path` itself when
//   `path` and `root` are both under different disk drives.
pub fn path_relative_to<P: AsRef<Path>>(path: &Path, root: P) -> Cow<'_, Path> {
	let root = root.as_ref();
	assert!(path.is_absolute());
	assert!(root.is_absolute());
	let mut p_comps = path.components();
	let mut r_comps = root.components();

	// 1. Ensure that the two paths have the same prefix.
	// 2. Strips any common prefix the two paths do have.
	//
	// NOTE:
	// Prefixes are platform dependent,
	// but different prefixes would for example indicate paths for different drives
	// on Windows.
	let (p_head, r_head) = loop {
		use std::path::Component::*;
		match (p_comps.next(), r_comps.next()) {
			(Some(RootDir), Some(RootDir)) => (),
			(Some(Prefix(a)), Some(Prefix(b))) if a == b => (),
			(Some(Prefix(_) | RootDir), _) | (_, Some(Prefix(_) | RootDir)) => {
				return Cow::from(path);
			}
			(None, None) => break (None, None),
			(a, b) if a != b => break (a, b),
			_ => (),
		}
	};

	let p_comps = p_head.into_iter().chain(p_comps);
	let walk_up = r_head.into_iter().chain(r_comps).map(|_| Component::ParentDir);

	let mut buf = PathBuf::new();
	buf.extend(walk_up);
	buf.extend(p_comps);

	Cow::from(buf)
}

#[inline]
pub fn optional_bool(s: &str) -> Option<bool> {
	if s == "true" {
		Some(true)
	} else if s == "false" {
		Some(false)
	} else {
		None
	}
}

#[cfg(test)]
mod tests {
	use std::{borrow::Cow, path::Path};

	use super::path_relative_to;

	#[cfg(unix)]
	#[test]
	fn test_path_relative_to() {
		let foo = "/foo";
		let bar = "/foo/bar";
		let baz = "/foo/bar/baz";
		let qux = "/foo/bar/qux";
		let aha = "/foo/aha/bar";

		assert_path_relate_to_root(bar, baz, "../"); // self reflection
		assert_path_relate_to_root(baz, bar, "baz"); // child entry
		assert_path_relate_to_root(qux, baz, "../qux"); // sibling entry
		assert_path_relate_to_root(baz, qux, "../baz"); // sibling entry
		assert_path_relate_to_root(foo, baz, "../../"); // ancestor entry
		assert_path_relate_to_root(aha, baz, "../../aha/bar"); // ancestor's child entry
	}

	fn assert_path_relate_to_root(path: &str, root: &str, res: &str) {
		assert_eq!(path_relative_to(Path::new(path), Path::new(root)), Cow::Borrowed(Path::new(res)));
	}
}
