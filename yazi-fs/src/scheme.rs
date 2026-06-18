use std::path::PathBuf;

use yazi_shared::{path::PathBufDyn, scheme::{AsScheme, Scheme, SchemeInventory, SchemeRef}};
use yazi_shim::mlua::UserDataFieldsExt;

use crate::Xdg;

pub trait FsScheme {
	fn cache(&self) -> Option<PathBuf>;
}

impl FsScheme for SchemeRef<'_> {
	fn cache(&self) -> Option<PathBuf> {
		match self {
			Self::Regular { .. } | Self::Search { .. } => None,
			Self::Archive { domain, .. } => Some(
				Xdg::temp_dir().join(format!("archive-{}", yazi_shared::scheme::Encode::domain(domain))),
			),
			Self::Sftp { domain, .. } => {
				Some(Xdg::temp_dir().join(format!("sftp-{}", yazi_shared::scheme::Encode::domain(domain))))
			}
		}
	}
}

impl FsScheme for Scheme {
	fn cache(&self) -> Option<PathBuf> { self.as_scheme().cache() }
}

// --- Inject
inventory::submit! {
	SchemeInventory {
		register: |registry| {
			registry.add_cached_field("cache", |_, me| Ok(me.cache().map(PathBufDyn::from)));
		}
	}
}
