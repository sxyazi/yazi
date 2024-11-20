use std::borrow::Cow;

use yazi_shared::{MIME_DIR, event::{Cmd, Data}};

use crate::manager::Manager;

struct Opt {
	skip: Option<usize>,
}

impl From<Cmd> for Opt {
	fn from(c: Cmd) -> Self { Self { skip: c.get("skip").and_then(Data::as_usize) } }
}

impl Manager {
	#[yazi_codegen::command]
	pub fn spot(&mut self, opt: Opt) {
		let Some(hovered) = self.hovered().cloned() else {
			return;
		};

		let mime = if hovered.is_dir() {
			Cow::Borrowed(MIME_DIR)
		} else {
			Cow::Owned(self.mimetype.get_owned(&hovered.url).unwrap_or_default())
		};

		if !self.active().spot.same_file(&hovered, &mime) {
			// self.active_mut().spot.reset();
		}

		if let Some(skip) = opt.skip {
			self.active_mut().spot.skip = skip;
		} else if !self.active().spot.same_url(&hovered.url) {
			self.active_mut().spot.skip = 0;
		}

		self.active_mut().spot.go(hovered, mime);
	}
}
