use std::{cell::Cell, ffi::OsStr, fs::{FileType, Metadata}, ops::Deref};

use anyhow::Result;
use tokio::fs;

use super::Location;
use crate::{fs::{Cha, ChaKind, Url}, theme::IconCache};

#[derive(Clone, Debug, Default)]
pub struct File {
	pub cha:     Cha,
	location:    Location,
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
		Self::from_meta(url, meta).await
	}

	#[inline]
	pub async fn from_meta(url: Url, meta: Metadata) -> Result<Self> {
		Self::from_loc(Location::from(url)?, meta).await
	}

	#[inline]
	pub fn from_dummy(url: Url, ft: Option<FileType>) -> Result<Self> {
		Ok(Self {
			cha:      ft.map_or_else(Cha::dummy, Cha::from),
			location: Location::from(url)?,
			link_to:  None,
			icon:     Default::default(),
		})
	}

	#[inline]
	pub async fn from_search(cwd: &Url, url: Url) -> Result<Self> {
		let loc = Location::from_search(cwd, url)?;
		let meta = fs::symlink_metadata(loc.url()).await?;
		Self::from_loc(loc, meta).await
	}

	async fn from_loc(loc: Location, mut meta: Metadata) -> Result<Self> {
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

		Ok(Self {
			cha: Cha::from(meta).with_kind(ck),
			location: loc,
			link_to,
			icon: Default::default(),
		})
	}
}

impl File {
	// --- Location
	#[inline]
	pub fn url(&self) -> &Url { self.location.url() }

	#[inline]
	pub fn url_owned(&self) -> Url { self.url().clone() }

	#[inline]
	pub fn name(&self) -> &OsStr { self.location.name() }

	#[inline]
	pub fn stem(&self) -> Option<&OsStr> { self.url().file_stem() }

	#[inline]
	pub fn parent(&self) -> Option<Url> { self.url().parent_url() }
}
