use std::path::PathBuf;

use yazi_shared::{auth::Auth, path::PathBufDyn, spec::SpecInventory};
use yazi_shim::{mlua::UserDataFieldsExt, strum::IntoStr};

use crate::{FsHash128, Xdg};

pub trait FsAuth {
	fn cache_root(&self) -> Option<PathBuf>;

	fn stamp_root(&self) -> Option<PathBuf> {
		self.cache_root().map(|mut root| {
			root.push("%stamp");
			root
		})
	}
}

impl FsAuth for Auth {
	fn cache_root(&self) -> Option<PathBuf> {
		if self.is_local() {
			return None;
		}

		Some(Xdg::temp_dir().join(format!(
			"{}_{}_{}",
			self.kind.into_str(),
			self.scheme,
			self.domain.hash_u128_str(&mut [0; 26])
		)))
	}
}

// --- Inject
inventory::submit! {
	SpecInventory {
		register: |registry| {
			registry.add_cached_field("cache", |_, me| Ok(me.cache_root().map(PathBufDyn::from)));
			registry.add_cached_field("stamp", |_, me| Ok(me.stamp_root().map(PathBufDyn::from)));
		}
	}
}
