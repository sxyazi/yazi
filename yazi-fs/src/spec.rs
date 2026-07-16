use std::path::PathBuf;

use yazi_shared::{auth::{Auth, AuthKind, EncodeAuth}, path::PathBufDyn, spec::SpecInventory};
use yazi_shim::{mlua::UserDataFieldsExt, strum::IntoStr};

use crate::Xdg;

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
					EncodeAuth::domain(&self.domain)
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
