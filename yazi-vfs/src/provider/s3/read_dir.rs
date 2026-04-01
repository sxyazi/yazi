use std::{collections::VecDeque, io, sync::Arc};

use object_store::{ListResult, ObjectMeta, path::Path};
use yazi_fs::provider::{DirReader, FileHolder};
use yazi_shared::{path::PathBufDyn, strand::StrandCow, url::{UrlBuf, UrlLike}};

use super::DynStore;

pub struct ReadDir {
	pub(super) dir:      Arc<UrlBuf>,
	pub(super) store:    DynStore,
	pub(super) prefix:   String,
	pub(super) token:    Option<String>,
	pub(super) finished: bool,
	pub(super) page_size: usize,
	pub(super) buffer:   VecDeque<DirEntry>,
}

impl DirReader for ReadDir {
	type Entry = DirEntry;

	async fn next(&mut self) -> io::Result<Option<Self::Entry>> {
		loop {
			if let Some(entry) = self.buffer.pop_front() {
				return Ok(Some(entry));
			}
			if self.finished {
				return Ok(None);
			}
			self.fetch_next_page().await?;
		}
	}
}

impl ReadDir {
	async fn fetch_next_page(&mut self) -> io::Result<()> {
		use object_store::list::{PaginatedListOptions, PaginatedListStore};

		let result = self
			.store
			.list_paginated(
				Some(&self.prefix),
				PaginatedListOptions {
					delimiter: Some("/".into()),
					max_keys: Some(self.page_size),
					page_token: self.token.take(),
					..Default::default()
				},
			)
			.await
			.map_err(super::s3::to_io)?;

		self.extend(result.result);
		self.token = result.page_token;
		self.finished = self.token.is_none();
		Ok(())
	}

	fn extend(&mut self, result: ListResult) {
		for prefix in result.common_prefixes {
			self.buffer.push_back(DirEntry {
				dir:  self.dir.clone(),
				name: basename(&prefix),
				kind: Kind::Dir,
			});
		}
		for object in result.objects {
			self.buffer.push_back(DirEntry {
				dir:  self.dir.clone(),
				name: basename(&object.location),
				kind: Kind::File(object),
			});
		}
	}
}

pub enum Kind {
	Dir,
	File(ObjectMeta),
}

pub struct DirEntry {
	pub(super) dir:  Arc<UrlBuf>,
	pub(super) name: String,
	pub(super) kind: Kind,
}

impl FileHolder for DirEntry {
	async fn file_type(&self) -> io::Result<yazi_fs::cha::ChaType> {
		Ok(match self.kind {
			Kind::Dir => yazi_fs::cha::ChaType::Dir,
			Kind::File(_) => yazi_fs::cha::ChaType::File,
		})
	}

	async fn metadata(&self) -> io::Result<yazi_fs::cha::Cha> {
		match &self.kind {
			Kind::Dir => Ok(super::metadata::dir(self.name.as_str())),
			Kind::File(meta) => Ok(super::metadata::object(self.name.as_str(), meta)),
		}
	}

	fn name(&self) -> StrandCow<'_> { self.name.as_str().into() }

	fn path(&self) -> PathBufDyn { self.url().into_loc() }

	fn url(&self) -> UrlBuf {
		self.dir.try_join(self.name.as_str()).expect("entry name is valid S3 path component")
	}
}

pub(super) fn basename(path: &Path) -> String {
	path.to_string().rsplit('/').next().unwrap_or_default().to_owned()
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn basename_returns_last_segment() {
		assert_eq!(basename(&Path::from("foo/bar/baz.txt")), "baz.txt");
		assert_eq!(basename(&Path::from("prefix/dir")), "dir");
	}
}
