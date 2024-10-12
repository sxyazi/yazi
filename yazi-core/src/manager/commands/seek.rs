use yazi_config::PLUGIN;
use yazi_plugin::isolate;
use yazi_shared::{MIME_DIR, event::{Cmd, Data}};

use crate::manager::Manager;

#[derive(Debug)]
struct Opt {
	units: i16,
}

impl From<Cmd> for Opt {
	fn from(c: Cmd) -> Self { Self { units: c.first().and_then(Data::as_i16).unwrap_or(0) } }
}

impl Manager {
	#[yazi_codegen::command]
	pub fn seek(&mut self, opt: Opt) {
		let Some(hovered) = self.hovered() else {
			return self.active_mut().preview.reset();
		};

		let mime = if hovered.is_dir() {
			MIME_DIR
		} else if let Some(s) = self.mimetype.get(&hovered.url) {
			s
		} else {
			return self.active_mut().preview.reset();
		};

		let Some(previewer) = PLUGIN.previewer(&hovered.url, mime) else {
			return self.active_mut().preview.reset();
		};

		isolate::seek_sync(&previewer.run, hovered.clone(), opt.units);
	}
}
