pub trait BytesExt {
	fn kebab_cased(&self) -> bool;
	fn split_by_seq(&self, sep: &[u8]) -> Option<(&[u8], &[u8])>;
}

impl BytesExt for [u8] {
	fn kebab_cased(&self) -> bool {
		self.iter().all(|&b| matches!(b, b'0'..=b'9' | b'a'..=b'z' | b'-'))
	}

	fn split_by_seq(&self, sep: &[u8]) -> Option<(&[u8], &[u8])> {
		let idx = memchr::memmem::find(self, sep)?;
		let (left, right) = self.split_at(idx);
		Some((left, &right[sep.len()..]))
	}
}
