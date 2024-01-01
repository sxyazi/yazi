use yazi_config::PLUGIN;
use yazi_plugin::isolate;
use yazi_shared::{event::Exec, render, MIME_DIR};

use crate::manager::Manager;

#[derive(Debug)]
pub struct Opt {
	units: i16,
}

impl From<&Exec> for Opt {
	fn from(e: &Exec) -> Self {
		Self { units: e.args.first().and_then(|s| s.parse().ok()).unwrap_or(0) }
	}
}

impl Manager {
	pub fn seek(&mut self, opt: impl Into<Opt>) {
		let Some(hovered) = self.hovered() else {
			return render!(self.active_mut().preview.reset());
		};

		let mime = if hovered.is_dir() {
			MIME_DIR
		} else if let Some(s) = self.mimetype.get(&hovered.url) {
			s
		} else {
			return render!(self.active_mut().preview.reset());
		};

		let Some(previewer) = PLUGIN.previewer(&hovered.url, mime) else {
			return render!(self.active_mut().preview.reset());
		};

		let opt = opt.into() as Opt;
		isolate::seek_sync(&previewer.exec, hovered.clone(), opt.units);
	}
}
