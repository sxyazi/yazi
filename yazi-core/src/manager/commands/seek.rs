use yazi_config::PLUGIN;
use yazi_plugin::isolate;
use yazi_shared::{event::{Cmd, Data}, render, MIME_DIR};

use crate::manager::Manager;

#[derive(Debug)]
pub struct Opt {
	units: i16,
}

impl From<Cmd> for Opt {
	fn from(c: Cmd) -> Self { Self { units: c.first().and_then(Data::as_i16).unwrap_or(0) } }
}

impl Manager {
	pub fn seek(&mut self, opt: impl Into<Opt>) {
		let Some(hovered) = self.hovered() else {
			return render!(self.active_mut().preview.reset());
		};

		let mime = if hovered.is_dir() {
			MIME_DIR
		} else if let Some(s) = self.mimetype.get(hovered.url()) {
			s
		} else {
			return render!(self.active_mut().preview.reset());
		};

		let Some(previewer) = PLUGIN.previewer(hovered.url(), mime) else {
			return render!(self.active_mut().preview.reset());
		};

		let opt = opt.into() as Opt;
		isolate::seek_sync(&previewer.run, hovered.clone(), opt.units);
	}
}
