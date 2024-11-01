use std::borrow::Cow;
use std::ffi::{OsStr, OsString};
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
			match opt.type_.as_str() {
				"path" => {
					match path_to_os_str(u, opt.separator) {
						Cow::Borrowed(p) => s.push(p),
						Cow::Owned(p) => s.push(&p),
					};
				}
				"dirname" => {
					if let Some(parent) = u.parent() {
						match path_to_os_str(parent, opt.separator) {
							Cow::Borrowed(p) => s.push(p),
							Cow::Owned(p) => s.push(&p),
						};
					}
				}
				"filename" => {
					s.push(u.name());
				}
				"name_without_ext" => {
					if let Some(stem) = u.file_stem() {
						s.push(stem);
					}
				}
				_ => return,
			}
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
fn path_to_os_str(path: &Path, _separator: PathSeparator) -> Cow<'_, OsStr> {
	return Cow::Borrowed(path.as_os_str());
}

#[cfg(target_os = "windows")]
fn path_to_os_str(path: &Path, separator: PathSeparator) -> Cow<'_, OsStr> {
	if let PathSeparator::Auto = separator {
		return Cow::Borrowed(path.as_os_str());
	};

	use std::path::Component;

	let mut s = OsString::with_capacity(path.as_os_str().len());
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

	return Cow::Owned(s);
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
			path_to_os_str(&path, PathSeparator::Auto).to_str(),
			Some("C:\\Users\\JohnDoe\\Downloads\\image.png"),
			"windows-auto",
		);
	}

	#[cfg(target_os = "windows")]
	#[test]
	fn test_path_to_os_str_windows_unix() {
		let path = PathBuf::from("C:\\Users\\JohnDoe\\Downloads\\image.png");
		assert_eq!(
			path_to_os_str(&path, PathSeparator::Unix).to_str(),
			Some("C:/Users/JohnDow/Downloads/image.png"),
			"windows-unix",
		);
	}

	#[cfg(not(target_os = "windows"))]
	#[test]
	fn test_path_to_os_str_unix_auto() {
		let path = PathBuf::from("/home/johndoe/Downloads/image.png");
		assert_eq!(
			path_to_os_str(&path, PathSeparator::Auto).to_str(),
			Some("/home/johndoe/Downloads/image.png"),
			"unix-auto"
		);
	}

	#[cfg(not(target_os = "windows"))]
	#[test]
	fn test_path_to_os_str_unix_unix() {
		let path = PathBuf::from("/home/johndoe/Downloads/image.png");
		assert_eq!(
			path_to_os_str(&path, PathSeparator::Unix).to_str(),
			Some("/home/johndoe/Downloads/image.png"),
			"unix-unix"
		);
	}
}
