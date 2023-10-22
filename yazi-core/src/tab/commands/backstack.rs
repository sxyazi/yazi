use crate::tab::Tab;

impl Tab {
	pub fn back(&mut self) -> bool {
		if let Some(url) = self.backstack.shift_backward().cloned() {
			futures::executor::block_on(self.cd(url));
		}
		false
	}

	pub fn forward(&mut self) -> bool {
		if let Some(url) = self.backstack.shift_forward().cloned() {
			futures::executor::block_on(self.cd(url));
		}
		false
	}
}
