use std::hash::{BuildHasher, Hash};

use data_encoding::BASE32_NOPAD;
use mlua::UserDataMethods;
use yazi_shared::{auth::Domain, url::{AsUrl, Url, UrlBuf, UrlBufInventory}};
use yazi_shim::Twox128;

use crate::{cha::Cha, file::FileSig};

pub trait FsHash64 {
	fn hash_u64(&self) -> u64;
}

impl FsHash64 for UrlBuf {
	fn hash_u64(&self) -> u64 { foldhash::fast::FixedState::default().hash_one(self.as_url()) }
}

impl FsHash64 for FileSig<'_> {
	fn hash_u64(&self) -> u64 { foldhash::fast::FixedState::default().hash_one(self) }
}

// Hash128
pub trait FsHash128 {
	fn hash_u128(&self) -> u128;

	fn hash_base32<'a>(&self, buf: &'a mut [u8; 26]) -> &'a str {
		BASE32_NOPAD.encode_mut_str(&self.hash_u128().to_be_bytes(), buf)
	}
}

impl FsHash128 for Url<'_> {
	fn hash_u128(&self) -> u128 {
		let mut h = Twox128::default();
		self.auth().hash(&mut h);
		for c in self.loc().components() {
			c.hash(&mut h);
		}
		h.finish_128()
	}
}

impl FsHash128 for UrlBuf {
	fn hash_u128(&self) -> u128 { self.as_url().hash_u128() }
}

impl FsHash128 for Domain<'_> {
	fn hash_u128(&self) -> u128 {
		let mut h = Twox128::default();
		self.hash(&mut h);
		h.finish_128()
	}
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

impl FsHash128 for FileSig<'_> {
	fn hash_u128(&self) -> u128 {
		let mut h = Twox128::default();
		self.hash(&mut h);
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
