use std::hash::Hash;

use mlua::{Function, Lua, Table};
use yazi_config::YAZI;
use yazi_fs::{FsHash128, file::{FileRef, FileSig}};
use yazi_shared::url::{Url, UrlBuf, UrlLike};
use yazi_shim::Twox128;

use super::Utils;

impl Utils {
	pub(super) fn file_cache(lua: &Lua) -> mlua::Result<Function> {
		struct Sig<'a>(FileSig<'a>, usize);

		impl FsHash128 for Sig<'_> {
			fn hash_u128(&self) -> u128 {
				let mut h = Twox128::default();
				self.0.hash(&mut h);
				self.1.hash(&mut h);
				h.finish_128()
			}
		}

		lua.create_function(|_, t: Table| {
			let file: FileRef = t.raw_get("file")?;
			file.borrow(|f| {
				if f.url.parent() == Some(Url::regular(&YAZI.preview.cache_dir)) {
					return Ok(None);
				}

				let sig = Sig(FileSig(f), t.raw_get("skip").unwrap_or_default());
				Ok(Some(UrlBuf::from(YAZI.preview.cache_dir.join(sig.hash_base32(&mut [0; 26])))))
			})
		})
	}
}
