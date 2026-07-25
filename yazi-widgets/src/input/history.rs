use std::mem;

#[derive(Debug, Default)]
pub struct InputHistory {
	pub name: String,

	pub(super) at:    Option<usize>,
	pub(super) draft: String,
}

impl InputHistory {
	pub(super) fn new(name: String) -> Self { Self { name, ..Default::default() } }

	pub fn take(&mut self) -> String {
		self.at = None;
		mem::take(&mut self.draft)
	}
}
