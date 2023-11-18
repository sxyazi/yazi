use std::{borrow::Cow, env, ffi::OsString, path::{Component, Path, PathBuf, MAIN_SEPARATOR}};

use tokio::fs;

use crate::Url;

#[inline]
pub fn current_cwd() -> Option<PathBuf> {
	env::var_os("PWD")
		.map(PathBuf::from)
		.filter(|p| p.is_absolute())
		.or_else(|| env::current_dir().ok())
}

fn _expand_path(p: &Path) -> PathBuf {
	// ${HOME} or $HOME
	#[cfg(unix)]
	let re = regex::Regex::new(r"\$(?:\{([^}]+)\}|([a-zA-Z\d_]+))").unwrap();

	// %USERPROFILE%
	#[cfg(windows)]
	let re = regex::Regex::new(r"%([^%]+)%").unwrap();

	let s = p.to_string_lossy();
	let s = re.replace_all(&s, |caps: &regex::Captures| {
		let name = caps.get(2).or_else(|| caps.get(1)).unwrap();
		env::var(name.as_str()).unwrap_or_else(|_| caps.get(0).unwrap().as_str().to_owned())
	});

	let p = Path::new(s.as_ref());
	if let Ok(rest) = p.strip_prefix("~") {
		#[cfg(unix)]
		let home = env::var_os("HOME");
		#[cfg(windows)]
		let home = env::var_os("USERPROFILE");

		return if let Some(p) = home { PathBuf::from(p).join(rest) } else { rest.to_path_buf() };
	}

	if p.is_absolute() {
		return p.to_path_buf();
	}
	current_cwd().map_or_else(|| p.to_path_buf(), |c| c.join(p))
}

#[inline]
pub fn expand_path(p: impl AsRef<Path>) -> PathBuf { _expand_path(p.as_ref()) }

#[inline]
pub fn ends_with_slash(p: &Path) -> bool {
	// TODO: uncomment this when Rust 1.74 is released
	// let b = p.as_os_str().as_encoded_bytes();
	// if let [.., last] = b { *last == MAIN_SEPARATOR as u8 } else { false }

	#[cfg(unix)]
	{
		use std::os::unix::ffi::OsStrExt;
		let b = p.as_os_str().as_bytes();
		if let [.., last] = b { *last == MAIN_SEPARATOR as u8 } else { false }
	}

	#[cfg(windows)]
	{
		let s = p.to_string_lossy();
		let b = s.as_bytes();
		if let [.., last] = b { *last == MAIN_SEPARATOR as u8 } else { false }
	}
}

pub async fn unique_path(mut p: Url) -> Url {
	let Some(stem) = p.file_stem().map(|s| s.to_owned()) else {
		return p;
	};

	let ext = p
		.extension()
		.map(|s| {
			let mut n = OsString::with_capacity(s.len() + 1);
			n.push(".");
			n.push(s);
			n
		})
		.unwrap_or_default();

	let mut i = 0;
	while fs::symlink_metadata(&p).await.is_ok() {
		i += 1;

		let mut name = OsString::with_capacity(stem.len() + ext.len() + 5);
		name.push(&stem);
		name.push(format!("_{i}"));
		if !ext.is_empty() {
			name.push(&ext);
		}

		p.set_file_name(name);
	}
	p
}

// Parameters
// * `path`: The absolute path(contains no `/./`) to get relative path.
// * `root`: The absolute path(contains no `/./`) to be compared.
//
// Return
// * Unix: The relative format to `root` of `path`.
// * Windows: The relative format to `root` of `path`; or `path` itself when
//   `path` and `root` are both under different disk drives.
pub fn path_relative_to<'a>(path: &'a Path, root: &Path) -> Cow<'a, Path> {
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

#[cfg(test)]
mod tests {
	use std::{borrow::Cow, path::Path};

	use super::path_relative_to;

	#[cfg(unix)]
	#[test]
	fn test_path_relative_to() {
		fn assert(path: &str, root: &str, res: &str) {
			assert_eq!(path_relative_to(Path::new(path), Path::new(root)), Cow::Borrowed(Path::new(res)));
		}

		assert("/a/b", "/a/b/c", "../");
		assert("/a/b/c", "/a/b", "c");
		assert("/a/b/c", "/a/b/d", "../c");
		assert("/a", "/a/b/c", "../../");
		assert("/a/a/b", "/a/b/b", "../../a/b");
	}

	#[cfg(windows)]
	#[test]
	fn test_path_relative_to() {
		fn assert(path: &str, root: &str, res: &str) {
			assert_eq!(path_relative_to(Path::new(path), Path::new(root)), Cow::Borrowed(Path::new(res)));
		}

		assert("C:\\a\\b", "C:\\a\\b\\c", "..\\");
		assert("C:\\a\\b\\c", "C:\\a\\b", "c");
		assert("C:\\a\\b\\c", "C:\\a\\b\\d", "..\\c");
		assert("C:\\a", "C:\\a\\b\\c", "..\\..\\");
		assert("C:\\a\\a\\b", "C:\\a\\b\\b", "..\\..\\a\\b");
	}
}
