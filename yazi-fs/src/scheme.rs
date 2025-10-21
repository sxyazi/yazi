use std::path::PathBuf;

use yazi_shared::scheme::{AsScheme, Scheme, SchemeRef};

use crate::Xdg;

pub trait FsScheme {
	fn cache(&self) -> Option<PathBuf>;
}

impl FsScheme for SchemeRef<'_> {
	fn cache(&self) -> Option<PathBuf> {
		match self {
			SchemeRef::Regular | SchemeRef::Search(_) => None,
			SchemeRef::Archive(name) => {
				Some(Xdg::cache_dir().join(format!("archive-{}", yazi_shared::url::Encode::domain(name))))
			}
			SchemeRef::Sftp(name) => {
				Some(Xdg::cache_dir().join(format!("sftp-{}", yazi_shared::url::Encode::domain(name))))
			}
		}
	}
}

impl FsScheme for Scheme {
	fn cache(&self) -> Option<PathBuf> { self.as_scheme().cache() }
}
