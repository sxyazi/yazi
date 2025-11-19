use std::{hash::{Hash, Hasher}, ops::Deref, path::Path};

use yazi_shared::{loc::Loc, path::{PathBufDyn, PathDyn, PathLike}, strand::Strand, url::{Url, UrlBuf, UrlLike}};

use crate::cha::{Cha, ChaType};

#[derive(Clone, Debug, Default)]
pub struct File {
	pub url:     UrlBuf,
	pub cha:     Cha,
	pub link_to: Option<PathBufDyn>,
}

impl Deref for File {
	type Target = Cha;

	fn deref(&self) -> &Self::Target { &self.cha }
}

impl File {
	#[inline]
	pub fn from_dummy(url: impl Into<UrlBuf>, r#type: Option<ChaType>) -> Self {
		let url = url.into();
		let cha = Cha::from_dummy(&url, r#type);
		Self { url, cha, link_to: None }
	}

	#[inline]
	pub fn chdir(&self, wd: &Path) -> Self {
		Self { url: self.url.rebase(wd), cha: self.cha, link_to: self.link_to.clone() }
	}
}

impl File {
	// --- Url
	#[inline]
	pub fn url_owned(&self) -> UrlBuf { self.url.clone() }

	#[inline]
	pub fn uri(&self) -> PathDyn<'_> { self.url.uri() }

	#[inline]
	pub fn urn(&self) -> PathDyn<'_> { self.url.urn() }

	#[inline]
	pub fn name(&self) -> Option<Strand<'_>> { self.url.name() }

	#[inline]
	pub fn stem(&self) -> Option<Strand<'_>> { self.url.stem() }

	pub fn link_to_url(&self) -> Option<Url<'_>> {
		let to = self.link_to.as_ref()?;
		let kind = self.url.kind();
		Some(match &self.url {
			UrlBuf::Regular(_) => Url::Regular(Loc::bare(to.as_os().ok()?)),
			UrlBuf::Search { domain, .. } => {
				Url::Search { loc: Loc::saturated(to.as_os().ok()?, kind), domain }
			}
			UrlBuf::Archive { domain, .. } => {
				Url::Archive { loc: Loc::saturated(to.as_os().ok()?, kind), domain }
			}
			UrlBuf::Sftp { domain, .. } => {
				Url::Sftp { loc: Loc::saturated(to.as_os().ok()?, kind), domain }
			}
		})
	}
}

impl Hash for File {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.url.hash(state);
		self.cha.len.hash(state);
		self.cha.btime.hash(state);
		self.cha.ctime.hash(state);
		self.cha.mtime.hash(state);
	}
}
