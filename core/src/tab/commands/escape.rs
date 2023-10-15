use crate::tab::{Mode, Tab};

impl Tab {
	pub fn escape(&mut self) -> bool {
		if self.finder.take().is_some() {
			return true;
		}

		if let Some((_, indices)) = self.mode.visual() {
			self.current.files.select_index(indices, Some(self.mode.is_select()));
			self.mode = Mode::Normal;
			return true;
		}

		if self.select_all(Some(false)) {
			return true;
		}

		self.search_stop()
	}
}
