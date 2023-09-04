use std::{collections::BTreeMap, path::{Path, PathBuf}};

use super::File;

#[derive(Debug)]
pub enum FilesOp {
	Read(PathBuf, Vec<File>),
	Size(PathBuf, BTreeMap<PathBuf, u64>),
	IOErr(PathBuf),
}

impl FilesOp {
	#[inline]
	pub fn path(&self) -> PathBuf {
		match self {
			Self::Read(path, _) => path,
			Self::Size(path, _) => path,
			Self::IOErr(path) => path,
		}
		.clone()
	}

	#[inline]
	pub fn clear(path: &Path) -> Self { Self::Read(path.to_path_buf(), Vec::new()) }
}
