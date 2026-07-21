use std::path::PathBuf;

use yazi_shared::{auth::{Auth, AuthKind}, path::PathBufDyn, spec::SpecInventory};
use yazi_shim::{mlua::UserDataFieldsExt, strum::IntoStr};

use crate::{FsHash128, Xdg};

pub trait FsSpec {
	fn cache(&self) -> Option<PathBuf>;
}

impl FsSpec for Auth {
	fn cache(&self) -> Option<PathBuf> {
		match self.kind {
			AuthKind::Regular | AuthKind::Search => None,
			AuthKind::Mount | AuthKind::Hub | AuthKind::Scope | AuthKind::Sftp => {
				Some(Xdg::temp_dir().join(format!(
					"{}_{}_{}",
					self.kind.into_str(),
					self.scheme,
					self.domain.hash_base32(&mut [0; 26])
				)))
			}
		}
	}
}

// --- Inject
inventory::submit! {
	SpecInventory {
		register: |registry| {
			registry.add_cached_field("cache", |_, me| Ok(me.cache().map(PathBufDyn::from)));
		}
	}
}
