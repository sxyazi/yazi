use std::{borrow::Cow, env, ffi::OsString, path::{Component, Path, PathBuf, MAIN_SEPARATOR}};

use super::maybe_exists;
use crate::fs::Url;

#[inline]
pub fn current_cwd() -> Option<PathBuf> {
	env::var_os("PWD")
		.map(PathBuf::from)
		.filter(|p| p.is_absolute())
		.or_else(|| env::current_dir().ok())
}

/// Resolves any `.` and `..` according to Unix and Windows path resolution
/// rules.
///
/// Return a simplified, canonical absolute path
fn resolve_relative_paths(p: PathBuf) -> PathBuf {
	assert!(p.is_absolute());
	let mut components = Vec::new();
	let mut prefix = None;

	for component in p.components() {
		match component {
			std::path::Component::CurDir => {
				// ignore `.`
			}
			std::path::Component::ParentDir => {
				// `..` case
				if components.len() > 0 && components[components.len() - 1] != Component::RootDir {
					components.pop();
				}
			}
			Component::RootDir => {
				// keep prefix on windows
				if prefix.is_none() {
					components.clear();
				}
				components.push(component);
			}
			Component::Prefix(_) => {
				prefix = Some(component);
				components.clear();
				components.push(component);
			}
			_ => {
				// add other paths
				components.push(component);
			}
		}
	}

	components.iter().collect()
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

	// Windows paths that only have a drive letter but no root, e.g. "D:"
	#[cfg(windows)]
	if s.len() == 2 {
		let b = s.as_bytes();
		if b[1] == b':' && b[0].is_ascii_alphabetic() {
			return PathBuf::from(s.to_uppercase() + "\\");
		}
	}

	let p = Path::new(s.as_ref());
	if let Ok(rest) = p.strip_prefix("~") {
		return dirs::home_dir().unwrap_or_default().join(rest);
	}

	if p.is_absolute() {
		return resolve_relative_paths(p.to_path_buf());
	}

	current_cwd().map_or_else(|| p.to_path_buf(), |c: PathBuf| resolve_relative_paths(c.join(p)))
}

#[inline]
pub fn expand_path(p: impl AsRef<Path>) -> PathBuf { _expand_path(p.as_ref()) }

#[inline]
pub fn ends_with_slash(p: &Path) -> bool {
	let b = p.as_os_str().as_encoded_bytes();
	if let [.., last] = b { *last == MAIN_SEPARATOR as u8 } else { false }
}

// FIXME: should return a `std::io::Result` to handle errors such as
// permission denied
pub async fn unique_name(mut u: Url) -> Url {
	let Some(stem) = u.file_stem().map(|s| s.to_owned()) else {
		return u;
	};

	let ext = u
		.extension()
		.map(|s| {
			let mut n = OsString::with_capacity(s.len() + 1);
			n.push(".");
			n.push(s);
			n
		})
		.unwrap_or_default();

	let mut i = 1u64;
	while maybe_exists(&u).await {
		let mut name = OsString::with_capacity(stem.len() + ext.len() + 5);
		name.push(&stem);
		name.push("_");
		name.push(i.to_string());
		name.push(&ext);

		u.set_file_name(name);
		i += 1;
	}
	u
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
	use std::{borrow::Cow, path::{Path, PathBuf}};

	use super::{path_relative_to, resolve_relative_paths};

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

	#[cfg(unix)]
	#[test]
	fn test_resolve_relative_paths() {
		fn assert(test_path: &str, res_path: &str) {
			assert_eq!(resolve_relative_paths(PathBuf::from(test_path)), PathBuf::from(res_path));
		}

		assert("/home/user/.././../etc/./config", "/etc/config");
		assert("/home/./user/./documents/..", "/home/user");
		assert("/./home/user/../etc", "/home/etc");
		assert("/../home/user", "/home/user");
		assert("/.././.././../home/user", "/home/user");
		assert("/../../.././../home/user", "/home/user");
		assert("/home/user/../../../../", "/");
	}

	#[cfg(windows)]
	#[test]
	fn test_resolve_relative_paths() {
		fn assert(test_path: &str, res_path: &str) {
			assert_eq!(resolve_relative_paths(PathBuf::from(test_path)), PathBuf::from(res_path));
		}

		assert(r"C:\Users\user\..\..\Windows\.\System32", r"C:\Windows\System32");
		assert(r"C:\Users\.\user\documents\..", r"C:\Users\user");
		assert(r"C:\.\Users\user\..\etc", r"C:\Users\etc");
		assert(r"C:\..\Users\user", r"C:\Users\user");
		assert(r"C:\..\.\..\.\..\Users\user", r"C:\Users\user");
		assert(r"C:\..\..\..\.\..\Users\user", r"C:\Users\user");
		assert(r"C:\Users\user\..\..\..\..", r"C:\");
	}
}
