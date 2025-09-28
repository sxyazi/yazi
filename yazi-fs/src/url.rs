use std::path::PathBuf;

use twox_hash::XxHash3_128;
use yazi_shared::{scheme::SchemeRef, url::{Url, UrlBuf}};

use crate::Xdg;

pub trait FsUrl {
	fn cache(&self) -> Option<PathBuf>;
}

impl FsUrl for Url<'_> {
	fn cache(&self) -> Option<PathBuf> {
		match self.scheme {
			SchemeRef::Regular | SchemeRef::Search(_) => None,
			SchemeRef::Archive(name) => Some(
				Xdg::cache_dir()
					.join(format!("archive-{}", yazi_shared::url::Encode::domain(name)))
					.join(format!("{:x}", XxHash3_128::oneshot(self.loc.bytes()))),
			),
			SchemeRef::Sftp(name) => Some(
				Xdg::cache_dir()
					.join(format!("sftp-{}", yazi_shared::url::Encode::domain(name)))
					.join(format!("{:x}", XxHash3_128::oneshot(self.loc.bytes()))),
			),
		}
	}
}

impl FsUrl for UrlBuf {
	fn cache(&self) -> Option<PathBuf> { self.as_url().cache() }
}
