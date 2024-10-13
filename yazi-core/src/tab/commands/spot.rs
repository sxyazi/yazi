use yazi_shared::event::{Cmd, Data};

use crate::tab::Tab;

struct Opt {
	skip: Option<usize>,
}

impl From<Cmd> for Opt {
	fn from(c: Cmd) -> Self { Self { skip: c.first().and_then(Data::as_usize) } }
}

impl Tab {
	#[yazi_codegen::command]
	pub fn spot(&mut self, c: Cmd) {
		let Some(hovered) = self.hovered().cloned() else {
			return self.preview.reset();
		};
	}
}
