use std::{borrow::Cow, env, ffi::OsString, future::Future, io, path::{Component, Path, PathBuf}};

use tokio::fs;
use yazi_shared::url::{Loc, Url};

use crate::CWD;

#[inline]
pub fn clean_url(url: &Url) -> Url { Url::from(clean_path(url)) }

#[inline]
pub fn clean_path(path: impl AsRef<Path>) -> PathBuf { _clean_path(path.as_ref()) }

fn _clean_path(path: &Path) -> PathBuf {
	let mut out = vec![];
	for c in path.components() {
		match c {
			Component::CurDir => {}
			Component::ParentDir => match out.last() {
				Some(Component::RootDir) => {}
				Some(Component::Normal(_)) => _ = out.pop(),
				None
				| Some(Component::CurDir)
				| Some(Component::ParentDir)
				| Some(Component::Prefix(_)) => out.push(c),
			},
			c => out.push(c),
		}
	}

	if out.is_empty() { PathBuf::from(".") } else { out.iter().collect() }
}

#[inline]
pub fn expand_path(p: impl AsRef<Path>) -> PathBuf { _expand_path(p.as_ref()) }

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
		clean_path(dirs::home_dir().unwrap_or_default().join(rest))
	} else if p.is_absolute() {
		clean_path(p)
	} else {
		clean_path(CWD.load().join(p))
	}
}

pub fn skip_path(p: &Path, u: usize) -> &Path {
	let mut it = p.components();
	for _ in 0..u {
		if it.next().is_none() {
			return Path::new("");
		}
	}
	it.as_path()
}

pub async fn unique_name<F>(u: Url, append: F) -> io::Result<Url>
where
	F: Future<Output = bool>,
{
	match fs::symlink_metadata(&u).await {
		Ok(_) => _unique_name(u, append.await).await,
		Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(u),
		Err(e) => Err(e),
	}
}

async fn _unique_name(mut u: Url, append: bool) -> io::Result<Url> {
	let Some(stem) = u.file_stem().map(|s| s.to_owned()) else {
		return Err(io::Error::new(io::ErrorKind::InvalidInput, "empty file stem"));
	};

	let dot_ext = u.extension().map_or_else(OsString::new, |e| {
		let mut s = OsString::with_capacity(e.len() + 1);
		s.push(".");
		s.push(e);
		s
	});

	let mut i = 1u64;
	let mut p = u.to_path();
	loop {
		let mut name = OsString::with_capacity(stem.len() + dot_ext.len() + 5);
		name.push(&stem);

		if append {
			name.push(&dot_ext);
			name.push("_");
			name.push(i.to_string());
		} else {
			name.push("_");
			name.push(i.to_string());
			name.push(&dot_ext);
		}

		p.set_file_name(name);
		match fs::symlink_metadata(&p).await {
			Ok(_) => i += 1,
			Err(e) if e.kind() == io::ErrorKind::NotFound => break,
			Err(e) => return Err(e),
		}
	}

	u.set_loc(Loc::from(u.base(), p));
	Ok(u)
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

#[cfg(windows)]
pub fn backslash_to_slash(p: &Path) -> Cow<Path> {
	let bytes = p.as_os_str().as_encoded_bytes();

	// Fast path to skip if there are no backslashes
	let skip_len = bytes.iter().take_while(|&&b| b != b'\\').count();
	if skip_len >= bytes.len() {
		return Cow::Borrowed(p);
	}

	let (skip, rest) = bytes.split_at(skip_len);
	let mut out = Vec::new();
	out.try_reserve_exact(bytes.len()).unwrap_or_else(|_| panic!());
	out.extend(skip);

	for &b in rest {
		out.push(if b == b'\\' { b'/' } else { b });
	}
	Cow::Owned(PathBuf::from(unsafe { OsString::from_encoded_bytes_unchecked(out) }))
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
