use std::ffi::{OsStr, OsString};
#[cfg(target_os = "windows")]
use std::path::Component;
use std::path::Path;

use yazi_plugin::CLIPBOARD;
use yazi_shared::event::Cmd;

use crate::tab::Tab;

struct Opt {
	type_: String,
	separator: PathSeparator,
}

impl From<Cmd> for Opt {
	fn from(mut c: Cmd) -> Self {
		Self { type_: c.take_first_str().unwrap_or_default(), separator: PathSeparator::from(&c) }
	}
}

impl Tab {
	#[yazi_codegen::command]
	pub fn copy(&mut self, opt: Opt) {
		if !self.try_escape_visual() {
			return;
		}

		let mut s = OsString::new();
		let mut it = self.selected_or_hovered(true).peekable();
		while let Some(u) = it.next() {
			s.push(match opt.type_.as_str() {
				"path" => path_to_os_str(u, opt.separator),
				"dirname" => u.parent().map_or(OsStr::new(""), |p| path_to_os_str(p, opt.separator)),
				"filename" => u.name(),
				"name_without_ext" => u.file_stem().unwrap_or(OsStr::new("")),
				_ => return,
			});
			if it.peek().is_some() {
				s.push("\n");
			}
		}

		// Copy the CWD path regardless even if the directory is empty
		if s.is_empty() && opt.type_ == "dirname" {
			s.push(self.cwd());
		}

		futures::executor::block_on(CLIPBOARD.set(s));
	}
}

#[derive(Default, Clone, Copy)]
enum PathSeparator {
	Unix,
	#[default]
	Auto,
}

impl From<&Cmd> for PathSeparator {
	fn from(c: &Cmd) -> Self {
		match c.str("separator") {
			Some("unix") => PathSeparator::Unix,
			Some("auto") => PathSeparator::Auto,
			_ => Default::default(),
		}
	}
}

#[cfg(not(target_os = "windows"))]
fn path_to_os_str(path: &Path, _separator: PathSeparator) -> &OsStr {
	return path.as_os_str();
}

#[cfg(target_os = "windows")]
fn path_to_os_str(path: &Path, separator: PathSeparator) -> &OsStr {
	if let PathSeparator::Auto = separator {
		return path.as_os_str();
	};

	let mut s = OsString::new();
	for component in path.components() {
		match component {
			Component::RootDir => {}
			Component::CurDir => s.push("."),
			Component::ParentDir => s.push(".."),
			Component::Normal(path) => s.push(path),
			Component::Prefix(prefix) => {
				// "C:\foo" => [Prefix("C:"), RootDir, Normal(foo)]
				s.push(prefix.as_os_str());
				// If we push a "/" below, we will met a RootDir and push a "/"
				// again resulting in "C://". So we need to skip that.
				continue;
			}
		};
		s.push("/");
	}

	return s.as_os_str();
}

#[cfg(test)]
mod tests {
	use std::path::PathBuf;

	use super::*;

	#[cfg(target_os = "windows")]
	#[test]
	fn test_path_to_os_str_windows_auto() {
		let path = PathBuf::from("C:\\Users\\JohnDoe\\Downloads\\image.png");
		assert_eq!(
			path_to_os_str(&path, PathSeparator::Auto),
			"C:\\Users\\JohnDoe\\Downloads\\image.png",
			"windows-auto",
		);
	}

	#[cfg(target_os = "windows")]
	#[test]
	fn test_path_to_os_str_windows_unix() {
		let path = PathBuf::from("C:\\Users\\JohnDoe\\Downloads\\image.png");
		assert_eq!(
			path_to_os_str(&path, PathSeparator::Unix),
			"C:/Users/JohnDow/Downloads/image.png",
			"windows-unix",
		);
	}

	#[cfg(not(target_os = "windows"))]
	#[test]
	fn test_path_to_os_str_unix_auto() {
		let path = PathBuf::from("/home/johndoe/Downloads/image.png");
		assert_eq!(
			path_to_os_str(&path, PathSeparator::Auto),
			"/home/johndoe/Downloads/image.png",
			"unix-auto"
		);
	}

	#[cfg(not(target_os = "windows"))]
	#[test]
	fn test_path_to_os_str_unix_unix() {
		let path = PathBuf::from("/home/johndoe/Downloads/image.png");
		assert_eq!(
			path_to_os_str(&path, PathSeparator::Unix),
			"/home/johndoe/Downloads/image.png",
			"unix-unix"
		);
	}
}
