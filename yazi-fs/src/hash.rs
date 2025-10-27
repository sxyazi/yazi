use std::hash::{BuildHasher, Hash};

use yazi_shared::url::AsUrl;
use yazi_shim::Twox128;

use crate::{File, cha::Cha};

pub trait FsHash64 {
	fn hash_u64(&self) -> u64;
}

impl FsHash64 for File {
	fn hash_u64(&self) -> u64 { foldhash::fast::FixedState::default().hash_one(self) }
}

impl<T: AsUrl> FsHash64 for T {
	fn hash_u64(&self) -> u64 { foldhash::fast::FixedState::default().hash_one(self.as_url()) }
}

// Hash128
pub trait FsHash128 {
	fn hash_u128(&self) -> u128;
}

impl FsHash128 for Cha {
	fn hash_u128(&self) -> u128 {
		let mut h = Twox128::default();

		self.len.hash(&mut h);
		self.btime_dur().ok().map(|d| d.as_nanos()).hash(&mut h);
		self.mtime_dur().ok().map(|d| d.as_nanos()).hash(&mut h);

		h.finish_128()
	}
}

impl<T: AsUrl> FsHash128 for T {
	fn hash_u128(&self) -> u128 {
		let mut h = Twox128::default();
		self.as_url().hash(&mut h);
		h.finish_128()
	}
}
