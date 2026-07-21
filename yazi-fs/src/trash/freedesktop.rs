use std::{ffi::OsStr, fs, hash::Hash, io, path::{Path, PathBuf}};

use hashbrown::HashMap;
use trash::{TrashItem, os_limited};
use yazi_macro::ok_or_not_found;
use yazi_shim::Twox128;

use super::{TrashCha, TrashEntry, TrashNode, TrashNodes};
use crate::{cha::{Cha, ChaSig}, file::File};

pub struct Trash;

impl Trash {
	pub fn new() -> io::Result<Self> { Ok(Self) }

	pub fn list(&self, node: Option<&TrashNode>) -> io::Result<Vec<TrashEntry>> {
		let Some(node) = node else {
			return os_limited::list()
				.map_err(io::Error::other)?
				.into_iter()
				.map(|item| TrashEntry::new(Self::path(&item.id)?, item.name, item.id))
				.collect();
		};

		fs::read_dir(self.resolve(node)?)?
			.map(|entry| {
				let entry = entry?;
				let name = entry.file_name();
				let path = entry.path();
				TrashEntry::new(path, name.clone(), name)
			})
			.collect()
	}

	pub fn entry(&self, node: &TrashNode) -> io::Result<TrashEntry> {
		let path = self.resolve(node)?;
		let name = path
			.file_name()
			.ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "invalid trash item path"))?
			.to_owned();
		TrashEntry::new(path, name, node.key.clone())
	}

	pub fn metadata(&self, node: &TrashNode, follow: bool) -> io::Result<Cha> {
		let path = self.resolve(node)?;
		if let Some(name) = node.rel.file_name() {
			Cha::from_trash(&path, name, follow)
		} else {
			let item = self.top_item(&node.top)?;
			Cha::from_trash(&path, &item.name, follow)
		}
	}

	pub(super) fn revalidate(
		&self,
		node: Option<&TrashNode>,
		current: &File,
	) -> io::Result<Option<File>> {
		let latest = if let Some(node) = node {
			self.entry(node)?.into_file(&current.url)
		} else {
			let mut roots: Vec<_> = os_limited::trash_folders().unwrap_or_default().into_iter().collect();
			roots.sort_unstable();

			let mut h = Twox128::default();
			for root in roots {
				let meta = ok_or_not_found!(fs::metadata(root.join("info")), continue);
				let cha = Cha::new(root.file_name().unwrap_or_default(), meta);

				root.hash(&mut h);
				ChaSig(cha).hash(&mut h);
			}

			let hash = h.finish_128();
			File {
				cha: Cha { len: hash as u64 ^ (hash >> 64) as u64, ..Cha::from_mold(true) },
				..current.clone()
			}
		};

		let changed = !latest.cha.hits(current.cha)
			|| latest.extra.link_to() != current.extra.link_to()
			|| latest.extra.backing() != current.extra.backing();

		Ok(changed.then_some(latest))
	}

	pub fn remove_file(&self, node: &TrashNode) -> io::Result<()> {
		if node.rel.as_os_str().is_empty() {
			os_limited::purge_all([self.top_item(&node.top)?]).map_err(io::Error::other)
		} else {
			fs::remove_file(self.resolve(node)?)
		}
	}

	pub fn remove_dir(&self, node: &TrashNode) -> io::Result<()> {
		if node.rel.as_os_str().is_empty() {
			os_limited::purge_all([self.top_item(&node.top)?]).map_err(io::Error::other)
		} else {
			fs::remove_dir(self.resolve(node)?)
		}
	}

	pub fn restore(&self, nodes: TrashNodes) -> io::Result<()> {
		let mut tops = Vec::new();
		let items: HashMap<_, _> = os_limited::list()
			.map_err(io::Error::other)?
			.into_iter()
			.map(|item| (item.id.clone(), item))
			.collect();

		for node in nodes {
			let item = items
				.get(&node.top)
				.ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "trash item no longer exists"))?;
			if node.rel.as_os_str().is_empty() {
				tops.push(item.clone());
				continue;
			}

			let from = self.resolve(&node)?;
			let to = item.original_path().join(node.rel);

			let is_dir = fs::symlink_metadata(&from)?.is_dir();
			if let Some(parent) = to.parent() {
				fs::create_dir_all(parent)?;
			}

			match if is_dir { fs::create_dir(&to) } else { fs::File::create_new(&to).map(|_| ()) } {
				Ok(()) => fs::rename(from, to),
				Err(e) if e.kind() == io::ErrorKind::AlreadyExists => Err(io::Error::new(
					io::ErrorKind::AlreadyExists,
					format!("restore target already exists: {to:?}"),
				)),
				Err(e) => Err(e),
			}?;
		}

		os_limited::restore_all(tops).map_err(io::Error::other)
	}

	pub fn empty(&self) -> io::Result<()> {
		os_limited::purge_all(os_limited::list().map_err(io::Error::other)?).map_err(io::Error::other)
	}

	fn resolve(&self, node: &TrashNode) -> io::Result<PathBuf> {
		let path = Self::path(&node.top)?;
		Ok(if node.rel.as_os_str().is_empty() { path } else { path.join(&node.rel) })
	}

	fn top_item(&self, key: &OsStr) -> io::Result<TrashItem> {
		os_limited::list()
			.map_err(io::Error::other)?
			.into_iter()
			.find(|item| item.id == key)
			.ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "trash item no longer exists"))
	}

	fn path(key: &OsStr) -> io::Result<PathBuf> {
		let info = Path::new(key); // Path of the trash info file, e.g. "~/.local/share/Trash/info/filename.txt.trashinfo"
		let parent = info
			.parent()
			.and_then(|parent| parent.parent())
			.ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid trash item path"))?;
		let stem = info
			.file_stem()
			.ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid trash item path"))?;

		Ok(parent.join("files").join(stem))
	}
}
