use crate::tab::Tab;

impl Tab {
	pub fn back(&mut self) -> bool {
		if let Some(url) = self.backstack.shift_backward().cloned() {
			self.cd(url.into_dir());
		}
		false
	}

	pub fn forward(&mut self) -> bool {
		if let Some(url) = self.backstack.shift_forward().cloned() {
			self.cd(url.into_dir());
		}
		false
	}
}
