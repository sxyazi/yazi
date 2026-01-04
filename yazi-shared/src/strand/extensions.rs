use crate::strand::{AsStrand, Strand, StrandBuf, StrandLike, ToStrand};

// --- StrandJoin
pub trait AsStrandJoin {
	fn join(self, sep: Strand) -> StrandBuf;
}

impl<T> AsStrandJoin for T
where
	T: IntoIterator,
	T::Item: AsStrand,
{
	fn join(self, sep: Strand) -> StrandBuf {
		let mut kind = sep.kind();
		let mut buf = Vec::new();
		for (i, item) in self.into_iter().enumerate() {
			if i > 0 {
				buf.extend(sep.encoded_bytes());
			}

			let s = item.as_strand();
			buf.extend(s.encoded_bytes());

			if s.kind() > kind {
				kind = s.kind();
			}
		}

		unsafe { StrandBuf::from_encoded_bytes(kind, buf) }
	}
}

// --- ToStrandJoin
pub trait ToStrandJoin {
	fn join(self, sep: Strand) -> StrandBuf;
}

impl<T> ToStrandJoin for T
where
	T: IntoIterator,
	T::Item: ToStrand,
{
	fn join(self, sep: Strand) -> StrandBuf {
		let mut kind = sep.kind();
		let mut buf = Vec::new();
		for (i, item) in self.into_iter().enumerate() {
			if i > 0 {
				buf.extend(sep.encoded_bytes());
			}

			let s = item.to_strand();
			buf.extend(s.encoded_bytes());

			if s.kind() > kind {
				kind = s.kind();
			}
		}

		unsafe { StrandBuf::from_encoded_bytes(kind, buf) }
	}
}
