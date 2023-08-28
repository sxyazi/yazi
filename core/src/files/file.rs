use std::{borrow::Cow, fs::Metadata, path::{Path, PathBuf}};

use anyhow::Result;
use tokio::fs;

#[derive(Clone, Debug)]
pub struct File {
	pub path:      PathBuf,
	pub meta:      Metadata,
	pub length:    Option<u64>,
	pub link_to:   Option<PathBuf>,
	pub is_link:   bool,
	pub is_hidden: bool,
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
	#[inline]
	pub fn path(&self) -> &PathBuf { &self.path }

	#[inline]
	pub fn set_path(mut self, path: PathBuf) -> Self {
		self.path = path;
		self
	}

	#[inline]
	pub fn name(&self) -> Option<Cow<str>> { self.path.file_name().map(|s| s.to_string_lossy()) }
}
