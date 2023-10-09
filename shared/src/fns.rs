use std::{borrow::Cow, env, ffi::OsStr, fmt::Display, path::{Component, Path, PathBuf}};

use tokio::fs;

use crate::Url;

pub fn expand_path(p: impl AsRef<Path>) -> PathBuf {
	let mut p = p.as_ref();

	// expand the environment variable by calling the "echo" command, in linux case,
	// this also expands the '~' path
	#[cfg(target_os = "windows")]
	let expanded_path = match std::process::Command::new("cmd").args(&["/C", "echo"]).arg(p).output() {
		Ok(output) if output.status.success() => Some(
			String::from_utf8_lossy(&output.stdout)
				.trim_end()
				.trim_matches('"')
				.replace("\\\"", "\"")
				.to_string(),
		),
		_ => None,
	};
	#[cfg(not(target_os = "windows"))]
	let expanded_path = match std::process::Command::new("sh")
		.arg("-c")
		.arg(format!("echo \"{}\"", p.to_string_lossy().replace("\"", "\\\"")))
		.output()
	{
		Ok(output) if output.status.success() => {
			Some(String::from_utf8_lossy(&output.stdout).trim_end().to_string())
		}
		_ => None,
	};

	if let Some(s) = &expanded_path {
		p = Path::new(s);
	}

	// support '~' on Windows
	#[cfg(target_os = "windows")]
	if let Ok(p) = p.strip_prefix("~") {
		if let Ok(home) = env::var("USERPROFILE") {
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

pub struct ShortPath<'a> {
	pub prefix: &'a Path,
	pub name:   &'a OsStr,
}

impl Display for ShortPath<'_> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		if self.prefix == Path::new("") {
			return write!(f, "{}", self.name.to_string_lossy());
		}
		write!(f, "{}/{}", self.prefix.display(), self.name.to_string_lossy())
	}
}

pub fn short_path<'a>(p: &'a Path, base: &Path) -> ShortPath<'a> {
	let p = p.strip_prefix(base).unwrap_or(p);
	let mut parts = p.components();
	let name = parts.next_back().and_then(|p| match p {
		Component::Normal(p) => Some(p),
		_ => None,
	});
	ShortPath { prefix: parts.as_path(), name: name.unwrap_or_default() }
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
	use std::{borrow::Cow, env, path::Path};

	use super::path_relative_to;
	use crate::expand_path;

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

	#[test]
	fn test_expand_path() {
		let path_s = r#"a"b"#;
		assert_eq!(expand_path(path_s), env::current_dir().unwrap().join(std::path::Path::new(path_s)));

		let path_s = r#"a'b"#;
		assert_eq!(expand_path(path_s), env::current_dir().unwrap().join(std::path::Path::new(path_s)));

		let path_s = r#"a; x b"#;
		assert_eq!(expand_path(path_s), env::current_dir().unwrap().join(std::path::Path::new(path_s)));

		let path_s = r#"a && x b"#;
		assert_eq!(expand_path(path_s), env::current_dir().unwrap().join(std::path::Path::new(path_s)));
	}
}
