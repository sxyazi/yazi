use std::ffi::{OsStr, OsString};

use crate::{external, tab::Tab};

impl Tab {
	pub fn copy(&self, type_: &str) -> bool {
		let mut s = OsString::new();
		let mut it = self.selected().into_iter().peekable();
		while let Some(f) = it.next() {
			s.push(match type_ {
				"path" => f.url.as_os_str(),
				"dirname" => f.url.parent().map_or(OsStr::new(""), |p| p.as_os_str()),
				"filename" => f.name().unwrap_or(OsStr::new("")),
				"name_without_ext" => f.stem().unwrap_or(OsStr::new("")),
				_ => return false,
			});
			if it.peek().is_some() {
				s.push("\n");
			}
		}

		futures::executor::block_on(external::clipboard_set(s)).ok();
		false
	}
}
