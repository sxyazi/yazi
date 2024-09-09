use std::{cell::Cell, ffi::OsStr, fs::{FileType, Metadata}, ops::Deref, path::Path};

use anyhow::Result;
use tokio::fs;

use super::Loc;
use crate::{fs::{Cha, ChaKind, Url}, theme::IconCache};

#[derive(Clone, Debug, Default)]
pub struct File {
	pub loc:     Loc,
	pub cha:     Cha,
	pub link_to: Option<Url>,
	pub icon:    Cell<IconCache>,
}

impl Deref for File {
	type Target = Cha;

	#[inline]
	fn deref(&self) -> &Self::Target { &self.cha }
}

impl AsRef<File> for File {
	#[inline]
	fn as_ref(&self) -> &File { self }
}

impl File {
	#[inline]
	pub async fn from(url: Url) -> Result<Self> {
		let meta = fs::symlink_metadata(&url).await?;
		Ok(Self::from_meta(url, meta).await)
	}

	#[inline]
	pub async fn from_search(cwd: &Url, url: Url) -> Result<Self> {
		let loc = Loc::from_search(cwd, url);
		let meta = fs::symlink_metadata(loc.url()).await?;
		Ok(Self::from_loc(loc, meta).await)
	}

	#[inline]
	pub async fn from_meta(url: Url, meta: Metadata) -> Self {
		Self::from_loc(Loc::from(url), meta).await
	}

	#[inline]
	pub fn from_dummy(url: Url, ft: Option<FileType>) -> Self {
		Self {
			loc:     Loc::from(url),
			cha:     ft.map_or_else(Cha::dummy, Cha::from),
			link_to: None,
			icon:    Default::default(),
		}
	}

	#[inline]
	pub fn rebase(&self, parent: &Url) -> Self {
		Self {
			loc:     self.loc.rebase(parent),
			cha:     self.cha,
			link_to: self.link_to.clone(),
			icon:    Default::default(),
		}
	}

	async fn from_loc(loc: Loc, mut meta: Metadata) -> Self {
		let mut ck = ChaKind::empty();
		let (is_link, mut link_to) = (meta.is_symlink(), None);

		if is_link {
			meta = fs::metadata(loc.url()).await.unwrap_or(meta);
			link_to = fs::read_link(loc.url()).await.map(Url::from).ok();
		}

		if is_link && meta.is_symlink() {
			ck |= ChaKind::ORPHAN;
		} else if is_link {
			ck |= ChaKind::LINK;
		}

		#[cfg(unix)]
		if loc.url().is_hidden() {
			ck |= ChaKind::HIDDEN;
		}
		#[cfg(windows)]
		{
			use std::os::windows::fs::MetadataExt;
			if meta.file_attributes() & 2 != 0 {
				ck |= ChaKind::HIDDEN;
			}
		}

		Self { loc, cha: Cha::from(meta).with_kind(ck), link_to, icon: Default::default() }
	}
}

impl File {
	// --- Loc
	#[inline]
	pub fn url(&self) -> &Url { self.loc.url() }

	#[inline]
	pub fn url_owned(&self) -> Url { self.url().clone() }

	#[inline]
	pub fn urn(&self) -> &Path { self.loc.urn() }

	#[inline]
	pub fn name(&self) -> &OsStr { self.loc.name() }

	#[inline]
	pub fn stem(&self) -> Option<&OsStr> { self.url().file_stem() }

	#[inline]
	pub fn parent(&self) -> Option<Url> { self.url().parent_url() }
}
