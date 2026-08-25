#[derive(Clone, Copy, Debug)]
pub struct Loaded(u32);

impl Loaded {
	pub fn new(idx: u8, rev: u16) -> Self {
		debug_assert!(idx < 16);

		Self(u32::from(rev) << 16 | 1u32 << idx)
	}

	pub fn mark(&mut self, idx: u8, rev: u16) -> bool {
		debug_assert!(idx < 16);

		let (idx, rev) = (1u32 << idx, rev as u32);
		if self.0 >> 16 != rev {
			self.0 = rev << 16;
		} else if self.0 & idx != 0 {
			return false;
		}

		self.0 |= idx;
		true
	}

	pub(crate) fn clear(&mut self, idx: u8, rev: u16) {
		debug_assert!(idx < 16);

		if self.0 >> 16 == rev as u32 {
			self.0 &= !(1u32 << idx);
		}
	}
}
