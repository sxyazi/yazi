use std::hash::{BuildHasher, Hash};

use mlua::UserDataMethods;
use yazi_shared::url::{AsUrl, UrlBuf, UrlBufInventory};
use yazi_shim::Twox128;

use crate::{cha::Cha, file::File};

pub trait FsHash64 {
	fn hash_u64(&self) -> u64;
}

impl FsHash64 for File {
	fn hash_u64(&self) -> u64 { foldhash::fast::FixedState::default().hash_one(self) }
}

impl FsHash64 for UrlBuf {
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

// --- Inject
inventory::submit! {
	UrlBufInventory {
		register: |registry| {
			registry.add_method("hash", |_, me, long: bool| {
				Ok(if long {
					format!("{:x}", me.hash_u128())
				} else {
					format!("{:x}", me.hash_u64())
				})
			});
		}
	}
}
