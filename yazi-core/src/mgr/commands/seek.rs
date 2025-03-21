use yazi_config::YAZI;
use yazi_plugin::isolate;
use yazi_shared::event::{CmdCow, Data};

use crate::mgr::Mgr;

#[derive(Debug)]
struct Opt {
	units: i16,
}

impl From<CmdCow> for Opt {
	fn from(c: CmdCow) -> Self { Self { units: c.first().and_then(Data::as_i16).unwrap_or(0) } }
}

impl Mgr {
	#[yazi_codegen::command]
	pub fn seek(&mut self, opt: Opt) {
		let Some(hovered) = self.hovered() else {
			return self.active_mut().preview.reset();
		};

		let Some(mime) = self.mimetype.by_file(hovered) else {
			return self.active_mut().preview.reset();
		};

		let Some(previewer) = YAZI.plugin.previewer(&hovered.url, mime) else {
			return self.active_mut().preview.reset();
		};

		isolate::seek_sync(&previewer.run, hovered.clone(), opt.units);
	}
}
