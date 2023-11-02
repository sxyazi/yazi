use crate::manager::Manager;

impl Manager {
	pub fn yank(&mut self, cut: bool) -> bool {
		self.yanked.0 = cut;
		self.yanked.1 = self.selected().into_iter().map(|f| f.url()).collect();
		true
	}

	pub fn unyank(&mut self) -> bool {
		self.yanked = Default::default();
		true
	}
}
