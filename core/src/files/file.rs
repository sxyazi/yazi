use std::{borrow::Cow, fs::Metadata, path::{Path, PathBuf}};

use anyhow::Result;
use tokio::fs;

#[derive(Clone, Debug)]
pub struct File {
	pub(super) path:      PathBuf,
	pub(super) meta:      Metadata,
	pub(super) length:    Option<u64>,
	pub(super) link_to:   Option<PathBuf>,
	pub(super) is_link:   bool,
	pub(super) is_hidden: bool,
}

impl File {
	#[inline]
	pub async fn from(path: &Path) -> Result<File> {
		let meta = fs::metadata(path).await?;
		Ok(Self::from_meta(path, meta).await)
	}

	pub async fn from_meta(path: &Path, mut meta: Metadata) -> File {
		let is_link = meta.is_symlink();
		let mut link_to = None;

		if is_link {
			meta = fs::metadata(&path).await.unwrap_or(meta);
			link_to = fs::read_link(&path).await.ok();
		}

		let length = if meta.is_dir() { None } else { Some(meta.len()) };
		let is_hidden = path.file_name().map(|s| s.to_string_lossy().starts_with('.')).unwrap_or(false);
		File { path: path.to_path_buf(), meta, length, link_to, is_link, is_hidden }
	}
}

impl File {
	// --- Path
	#[inline]
	pub fn path(&self) -> &PathBuf { &self.path }

	#[inline]
	pub fn set_path(&mut self, path: PathBuf) { self.path = path; }

	#[inline]
	pub fn name(&self) -> Option<Cow<str>> { self.path.file_name().map(|s| s.to_string_lossy()) }

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
