use std::{borrow::Cow, ffi::OsStr, fs::Metadata, path::PathBuf};

use anyhow::Result;
use shared::Url;
use tokio::fs;

#[derive(Clone, Debug)]
pub struct File {
	pub(super) url:       Url,
	pub(super) meta:      Metadata,
	pub(super) length:    Option<u64>,
	pub(super) link_to:   Option<PathBuf>,
	pub(super) is_link:   bool,
	pub(super) is_hidden: bool,
}

impl File {
	#[inline]
	pub async fn from(url: Url) -> Result<Self> {
		let meta = fs::metadata(&url).await?;
		Ok(Self::from_meta(url, meta).await)
	}

	pub async fn from_meta(url: Url, mut meta: Metadata) -> Self {
		let is_link = meta.is_symlink();
		let mut link_to = None;

		if is_link {
			meta = fs::metadata(&url).await.unwrap_or(meta);
			link_to = fs::read_link(&url).await.ok();
		}

		let length = if meta.is_dir() { None } else { Some(meta.len()) };
		let is_hidden = url.file_name().map(|s| s.to_string_lossy().starts_with('.')).unwrap_or(false);
		Self { url, meta, length, link_to, is_link, is_hidden }
	}
}

impl File {
	// --- Path
	#[inline]
	pub fn url(&self) -> &Url { &self.url }

	#[inline]
	pub fn set_url(&mut self, url: Url) { self.url = url; }

	#[inline]
	pub fn url_owned(&self) -> Url { self.url.clone() }

	#[inline]
	pub fn url_os_str(&self) -> &OsStr { self.url.as_os_str() }

	#[inline]
	pub fn name(&self) -> Option<&OsStr> { self.url.file_name() }

	#[inline]
	pub fn name_display(&self) -> Option<Cow<str>> {
		self.url.file_name().map(|s| s.to_string_lossy())
	}

	#[inline]
	pub fn stem(&self) -> Option<&OsStr> { self.url.file_stem() }

	#[inline]
	pub fn parent(&self) -> Option<Url> { self.url.parent_url() }

	// --- Meta
	#[inline]
	pub fn meta(&self) -> &Metadata { &self.meta }

	#[inline]
	pub fn is_file(&self) -> bool { self.meta.is_file() }

	#[inline]
	pub fn is_dir(&self) -> bool { self.meta.is_dir() }

	// --- Length
	#[inline]
	pub fn length(&self) -> Option<u64> { self.length }

	// --- Link to
	#[inline]
	pub fn link_to(&self) -> Option<&PathBuf> { self.link_to.as_ref() }
}
