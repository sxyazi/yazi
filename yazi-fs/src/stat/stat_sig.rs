use std::{hash::{Hash, Hasher}, ops::Deref};

use crate::stat::Stat;

#[derive(Clone, Copy, Debug)]
pub struct StatSig(pub(crate) Stat);

impl Deref for StatSig {
	type Target = Stat;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl Hash for StatSig {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.len.hash(state);
		self.btime.hash(state);
		self.ctime.hash(state);
		self.mtime.hash(state);
	}
}
