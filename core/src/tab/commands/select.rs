use crate::tab::Tab;

impl Tab {
	pub fn select(&mut self, state: Option<bool>) -> bool {
		if let Some(u) = self.current.hovered().map(|h| h.url()) {
			return self.current.files.select(&u, state);
		}
		false
	}

	pub fn select_all(&mut self, state: Option<bool>) -> bool { self.current.files.select_all(state) }
}
