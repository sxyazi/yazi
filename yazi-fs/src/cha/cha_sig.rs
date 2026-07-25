use std::{hash::{Hash, Hasher}, ops::Deref};

use crate::cha::Cha;

#[derive(Clone, Copy, Debug)]
pub struct ChaSig(pub Cha);

impl Deref for ChaSig {
	type Target = Cha;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl Hash for ChaSig {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.len.hash(state);
		self.btime.hash(state);
		self.ctime.hash(state);
		self.mtime.hash(state);
	}
}
