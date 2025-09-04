pub trait BytesExt {
	fn split_by_seq(&self, sep: &[u8]) -> Option<(&[u8], &[u8])>;
}

impl BytesExt for [u8] {
	fn split_by_seq(&self, sep: &[u8]) -> Option<(&[u8], &[u8])> {
		let idx = memchr::memmem::find(self, sep)?;
		let (left, right) = self.split_at(idx);
		Some((left, &right[sep.len()..]))
	}
}
