use std::{ffi::OsString, path::Path};

use yazi_parser::tab::CopyOpt;
use yazi_widgets::CLIPBOARD;

use crate::tab::Tab;

impl Tab {
	#[yazi_codegen::command]
	pub fn copy(&mut self, opt: CopyOpt) {
		if !self.try_escape_visual() {
			return;
		}

		let mut s = OsString::new();
		let mut it = if opt.hovered {
			Box::new(self.hovered().map(|h| &h.url).into_iter())
		} else {
			self.selected_or_hovered()
		}
		.peekable();

		while let Some(u) = it.next() {
			s.push(match opt.r#type.as_ref() {
				"path" => opt.separator.transform(u),
				"dirname" => opt.separator.transform(u.parent().unwrap_or(Path::new(""))),
				"filename" => opt.separator.transform(u.name()),
				"name_without_ext" => opt.separator.transform(u.file_stem().unwrap_or_default()),
				_ => return,
			});
			if it.peek().is_some() {
				s.push("\n");
			}
		}

		// Copy the CWD path regardless even if the directory is empty
		if s.is_empty() && opt.r#type == "dirname" {
			s.push(self.cwd());
		}

		futures::executor::block_on(CLIPBOARD.set(s));
	}
}
