use std::collections::BTreeMap;

#[derive(Default)]
pub struct Completion {
	pub(super) caches: BTreeMap<String, Vec<String>>,
	pub(super) cands:  Vec<String>,
	pub(super) offset: usize,
	pub cursor:        usize,

	pub(super) ticket: usize,
	pub visible:       bool,
}

impl Completion {
	#[inline]
	pub fn window(&self) -> &[String] {
		let end = (self.offset + self.limit()).min(self.cands.len());
		&self.cands[self.offset..end]
	}

	#[inline]
	pub fn limit(&self) -> usize { self.cands.len().min(5) }

	#[inline]
	pub fn selected(&self) -> &String { &self.cands[self.cursor] }
}
