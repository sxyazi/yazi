use std::{borrow::Cow, env, path::{Component, Path, PathBuf}};

use tokio::fs;

use crate::Url;

pub fn expand_path(p: impl AsRef<Path>) -> PathBuf {
	// ${HOME} or $HOME
	#[cfg(unix)]
	let re = regex::Regex::new(r"\$(?:\{([^}]+)\}|([a-zA-Z\d_]+))").unwrap();

	// %USERPROFILE%
	#[cfg(windows)]
	let re = regex::Regex::new(r"%([^%]+)%").unwrap();

	let s = p.as_ref().to_string_lossy();
	let s = re.replace_all(&s, |caps: &regex::Captures| {
		let name = caps.get(2).or_else(|| caps.get(1)).unwrap();
		env::var(name.as_str()).unwrap_or_else(|_| caps.get(0).unwrap().as_str().to_owned())
	});

	let p = Path::new(s.as_ref());
	if let Ok(p) = p.strip_prefix("~") {
		#[cfg(unix)]
		if let Some(home) = env::var_os("HOME") {
			return Path::new(&home).join(p);
		}
		#[cfg(windows)]
		if let Some(home) = env::var_os("USERPROFILE") {
			return Path::new(&home).join(p);
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

pub async fn unique_path(mut p: Url) -> Url {
	let Some(name) = p.file_name().map(|n| n.to_owned()) else {
		return p;
	};

	let mut i = 0;
	while fs::symlink_metadata(&p).await.is_ok() {
		i += 1;
		let name_str = name.as_os_str().to_str().unwrap();
		let (base_name, extension) = split_filename_extension(name_str);
		p.set_file_name(format!("{base_name}_{i}{extension}"));
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

#[inline]
pub fn optional_bool(s: &str) -> Option<bool> {
	match s {
		"true" => Some(true),
		"false" => Some(false),
		_ => None,
	}
}

pub fn split_filename_extension(filename: &str) -> (&str, &str) {
	if let Some(dot_pos) = filename.rfind('.') {
		let (base_name, extension) = filename.split_at(dot_pos);
		(base_name, extension)
	} else {
		(filename, "")
	}
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
