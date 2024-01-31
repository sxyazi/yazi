use std::ffi::{OsStr, OsString};

use yazi_shared::event::Cmd;

use crate::{tab::Tab, CLIPBOARD};

pub struct Opt {
	type_: String,
}

impl From<Cmd> for Opt {
	fn from(mut c: Cmd) -> Self { Self { type_: c.take_first().unwrap_or_default() } }
}

impl Tab {
	pub fn copy(&self, opt: impl Into<Opt>) {
		let opt = opt.into() as Opt;

		let mut s = OsString::new();
		let mut it = self.selected().into_iter().peekable();
		while let Some(f) = it.next() {
			s.push(match opt.type_.as_str() {
				"path" => f.url.as_os_str(),
				"dirname" => f.url.parent().map_or(OsStr::new(""), |p| p.as_os_str()),
				"filename" => f.name().unwrap_or(OsStr::new("")),
				"name_without_ext" => f.stem().unwrap_or(OsStr::new("")),
				_ => return,
			});
			if it.peek().is_some() {
				s.push("\n");
			}
		}

		futures::executor::block_on(CLIPBOARD.set(s));
	}
}
