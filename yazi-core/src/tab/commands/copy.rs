use std::{borrow::Cow, ffi::{OsStr, OsString}, path::Path};

use yazi_plugin::CLIPBOARD;
use yazi_shared::event::CmdCow;

use crate::tab::Tab;

struct Opt {
	r#type:    Cow<'static, str>,
	separator: Separator,
	hovered:   bool,
}

impl From<CmdCow> for Opt {
	fn from(mut c: CmdCow) -> Self {
		Self {
			r#type:    c.take_first_str().unwrap_or_default(),
			separator: c.str("separator").unwrap_or_default().into(),
			hovered:   c.bool("hovered"),
		}
	}
}

impl Tab {
	#[yazi_codegen::command]
	pub fn copy(&mut self, opt: Opt) {
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

// --- Separator
#[derive(Clone, Copy, PartialEq, Eq)]
enum Separator {
	Auto,
	Unix,
}

impl From<&str> for Separator {
	fn from(value: &str) -> Self {
		match value {
			"unix" => Self::Unix,
			_ => Self::Auto,
		}
	}
}

impl Separator {
	fn transform<T: AsRef<Path> + ?Sized>(self, p: &T) -> Cow<'_, OsStr> {
		#[cfg(windows)]
		if self == Self::Unix {
			return match yazi_fs::backslash_to_slash(p.as_ref()) {
				Cow::Owned(p) => Cow::Owned(p.into_os_string()),
				Cow::Borrowed(p) => Cow::Borrowed(p.as_os_str()),
			};
		}
		Cow::Borrowed(p.as_ref().as_os_str())
	}
}
