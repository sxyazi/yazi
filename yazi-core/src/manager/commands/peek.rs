use crate::manager::Manager;

impl Manager {
	pub fn peek(&mut self, sequent: bool, show_image: bool) -> bool {
		let Some(hovered) = self.hovered().cloned() else {
			return self.active_mut().preview.reset(|_| true);
		};

		let url = &hovered.url;
		if !show_image {
			self.active_mut().preview.reset(|l| l.is_image());
		}

		if hovered.is_dir() {
			let position = self.active().history(url).map(|f| (f.offset, f.files.len()));
			self.active_mut().preview.folder(url, position, sequent);
			return false;
		}

		let Some(mime) = self.mimetype.get(url).cloned() else {
			return self.active_mut().preview.reset(|_| true);
		};

		if sequent {
			self.active_mut().preview.sequent(url, &mime, show_image);
		} else {
			self.active_mut().preview.go(url, &mime, show_image);
		}
		false
	}
}
