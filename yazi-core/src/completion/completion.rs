#[derive(Default)]
pub struct Completion {
	pub(super) items:  Vec<String>,
	pub(super) offset: usize,
	pub cursor:        usize,

	pub ticket:  usize,
	pub visible: bool,
}

impl Completion {
	#[inline]
	pub fn window(&self) -> &[String] {
		let end = (self.offset + self.limit()).min(self.items.len());
		&self.items[self.offset..end]
	}

	#[inline]
	pub fn limit(&self) -> usize { self.items.len().min(5) }
}
