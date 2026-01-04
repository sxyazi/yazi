use std::fmt::Display;

use crate::BytePredictor;

pub trait BytesExt {
	fn display(&self) -> impl Display;

	fn kebab_cased(&self) -> bool;

	fn rsplit_pred_once<P: BytePredictor>(&self, pred: P) -> Option<(&[u8], &[u8])>;

	fn rsplit_seq_once(&self, sep: &[u8]) -> Option<(&[u8], &[u8])>;

	fn split_seq_once(&self, sep: &[u8]) -> Option<(&[u8], &[u8])>;
}

impl BytesExt for [u8] {
	fn display(&self) -> impl Display {
		struct D<'a>(&'a [u8]);

		impl Display for D<'_> {
			fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
				for chunk in self.0.utf8_chunks() {
					chunk.valid().fmt(f)?;
					if !chunk.invalid().is_empty() {
						char::REPLACEMENT_CHARACTER.fmt(f)?;
					}
				}
				Ok(())
			}
		}

		D(self)
	}

	fn kebab_cased(&self) -> bool {
		self.iter().all(|&b| matches!(b, b'0'..=b'9' | b'a'..=b'z' | b'-'))
	}

	fn rsplit_pred_once<P: BytePredictor>(&self, pred: P) -> Option<(&[u8], &[u8])> {
		for (i, &byte) in self.iter().enumerate().rev() {
			if pred.predicate(byte) {
				let (a, b) = self.split_at(i);
				return Some((a, &b[1..]));
			}
		}
		None
	}

	fn split_seq_once(&self, sep: &[u8]) -> Option<(&[u8], &[u8])> {
		let idx = memchr::memmem::find(self, sep)?;
		let (a, b) = self.split_at(idx);
		Some((a, &b[sep.len()..]))
	}

	fn rsplit_seq_once(&self, sep: &[u8]) -> Option<(&[u8], &[u8])> {
		let idx = memchr::memmem::rfind(self, sep)?;
		let (a, b) = self.split_at(idx);
		Some((a, &b[sep.len()..]))
	}
}
