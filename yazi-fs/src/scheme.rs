use std::path::PathBuf;

use yazi_shared::scheme::{AsScheme, Scheme, SchemeRef};

use crate::Xdg;

pub trait FsScheme {
	fn cache(&self) -> Option<PathBuf>;
}

impl FsScheme for SchemeRef<'_> {
	fn cache(&self) -> Option<PathBuf> {
		match self {
			Self::Regular { .. } | Self::Search { .. } => None,
			Self::Archive { domain, .. } => Some(
				Xdg::cache_dir().join(format!("archive-{}", yazi_shared::scheme::Encode::domain(domain))),
			),
			Self::Sftp { domain, .. } => {
				Some(Xdg::cache_dir().join(format!("sftp-{}", yazi_shared::scheme::Encode::domain(domain))))
			}
		}
	}
}

impl FsScheme for Scheme {
	fn cache(&self) -> Option<PathBuf> { self.as_scheme().cache() }
}
