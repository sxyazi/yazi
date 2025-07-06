use yazi_config::YAZI;
use yazi_parser::mgr::SeekOpt;
use yazi_plugin::isolate;

use crate::mgr::Mgr;

impl Mgr {
	#[yazi_codegen::command]
	pub fn seek(&mut self, opt: SeekOpt) {
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
