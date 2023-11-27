use std::ffi::{OsStr, OsString};

use yazi_shared::event::Exec;

use crate::{external, tab::Tab};

pub struct Opt<'a> {
	type_: &'a str,
}

impl<'a> From<&'a Exec> for Opt<'a> {
	fn from(e: &'a Exec) -> Self { Self { type_: e.args.first().map(|s| s.as_str()).unwrap_or("") } }
}

impl Tab {
	pub fn copy<'a>(&self, opt: impl Into<Opt<'a>>) -> bool {
		let opt = opt.into() as Opt;

		let mut s = OsString::new();
		let mut it = self.selected().into_iter().peekable();
		while let Some(f) = it.next() {
			s.push(match opt.type_ {
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
