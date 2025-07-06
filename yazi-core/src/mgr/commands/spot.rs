use yazi_parser::mgr::SpotOpt;

use crate::mgr::Mgr;

impl Mgr {
	#[yazi_codegen::command]
	pub fn spot(&mut self, opt: SpotOpt) {
		let Some(hovered) = self.hovered().cloned() else {
			return;
		};

		let mime = self.mimetype.by_file_owned(&hovered).unwrap_or_default();
		// if !self.active().spot.same_file(&hovered, &mime) {
		// self.active_mut().spot.reset();
		// }

		if let Some(skip) = opt.skip {
			self.active_mut().spot.skip = skip;
		} else if !self.active().spot.same_url(&hovered.url) {
			self.active_mut().spot.skip = 0;
		}

		self.active_mut().spot.go(hovered, mime);
	}
}
