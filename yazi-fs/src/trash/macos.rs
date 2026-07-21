use std::{ffi::{OsStr, OsString}, fs, io, path::{Component, Path, PathBuf}};

use ds_parser::Value;
use hashbrown::HashMap;
use yazi_macro::ok_or_not_found;

use super::{TrashCha, TrashEntry, TrashNode, TrashNodes};
use crate::{cha::Cha, file::File};

pub struct Trash;

impl Trash {
	pub fn new() -> io::Result<Self> { Ok(Self) }

	pub fn list(&self, node: Option<&TrashNode>) -> io::Result<Vec<TrashEntry>> {
		let it = match fs::read_dir(self.resolve(node)?) {
			Ok(it) => it,
			Err(e) if e.kind() == io::ErrorKind::NotFound && node.is_none() => return Ok(vec![]),
			Err(e) => return Err(e),
		};

		it.map(|entry| {
			let entry = entry?;
			let name = entry.file_name();
			let path = entry.path();
			let key = if node.is_some() { name.clone() } else { path.as_os_str().to_owned() };
			TrashEntry::new(path, name, key)
		})
		.collect()
	}

	pub fn entry(&self, node: &TrashNode) -> io::Result<TrashEntry> {
		let path = self.resolve(Some(node))?;
		let name = path
			.file_name()
			.ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "invalid trash item path"))?
			.to_owned();
		TrashEntry::new(path, name, node.key.clone())
	}

	pub fn metadata(&self, node: &TrashNode, follow: bool) -> io::Result<Cha> {
		let path = self.resolve(Some(node))?;
		Cha::from_trash(&path, path.file_name().unwrap_or_default(), follow)
	}

	pub(super) fn revalidate(
		&self,
		node: Option<&TrashNode>,
		current: &File,
	) -> io::Result<Option<File>> {
		let latest = if let Some(node) = node {
			self.entry(node)?.into_file(&current.url)
		} else {
			let path = self.resolve(None)?;
			let cha = match fs::symlink_metadata(&path) {
				Ok(meta) => Cha::new(path.file_name().unwrap_or_default(), meta),
				Err(e) if e.kind() == io::ErrorKind::NotFound => Cha::from_mold(true),
				Err(e) => return Err(e),
			};
			File { cha, ..current.clone() }
		};

		let changed = !latest.cha.hits(current.cha)
			|| latest.extra.link_to() != current.extra.link_to()
			|| latest.extra.backing() != current.extra.backing();

		Ok(changed.then_some(latest))
	}

	pub fn remove_file(&self, node: &TrashNode) -> io::Result<()> {
		fs::remove_file(self.resolve(Some(node))?)
	}

	pub fn remove_dir(&self, node: &TrashNode) -> io::Result<()> {
		fs::remove_dir(self.resolve(Some(node))?)
	}

	pub fn restore(&self, nodes: TrashNodes) -> io::Result<()> {
		let root = self.resolve(None)?;
		let locations = OriginalLocation::parse(&fs::read(root.join(".DS_Store"))?)?;

		for node in nodes {
			let name = Path::new(&node.top).file_name().unwrap_or_default();
			let location = locations.get(name).ok_or_else(|| {
				io::Error::new(
					io::ErrorKind::NotFound,
					format!("original location is unavailable for trash item: {:?}", node.top),
				)
			})?;

			let from = self.resolve(Some(&node))?;
			let to = location.join(&node)?;

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

		Ok(())
	}

	pub fn empty(&self) -> io::Result<()> {
		let root = self.resolve(None)?;
		for entry in ok_or_not_found!(fs::read_dir(root), return Ok(())) {
			let entry = entry?;
			if entry.file_type()?.is_dir() {
				fs::remove_dir_all(entry.path())?;
			} else {
				fs::remove_file(entry.path())?;
			}
		}
		Ok(())
	}

	fn resolve(&self, node: Option<&TrashNode>) -> io::Result<PathBuf> {
		if let Some(node) = node {
			let top = Path::new(&node.top);
			return Ok(if node.rel.as_os_str().is_empty() { top.into() } else { top.join(&node.rel) });
		}

		Ok(
			dirs::home_dir()
				.filter(|p| p.is_absolute())
				.ok_or_else(|| {
					io::Error::other("cannot determine home directory for trash root resolution")
				})?
				.join(".Trash"),
		)
	}
}

// --- OriginalLocation
#[derive(Default)]
struct OriginalLocation {
	parent: Option<PathBuf>,
	name:   Option<OsString>,
}

impl OriginalLocation {
	fn parse(bytes: &[u8]) -> io::Result<HashMap<OsString, OriginalLocation>> {
		let store = ds_parser::parse(bytes).map_err(io::Error::other)?;
		let mut locations = HashMap::<OsString, Self>::new();

		for record in store.records {
			let Value::Ustr(value) = record.value else { continue };
			if value.is_empty() {
				continue;
			}

			let location = locations.entry_ref(OsStr::new(&record.name)).or_default();
			match &record.field.fourcc().bytes() {
				b"ptbL" => location.parent = Some(value.into()),
				b"ptbN" => location.name = Some(value.into()),
				_ => {}
			}
		}

		Ok(locations)
	}

	fn join(&self, node: &TrashNode) -> io::Result<PathBuf> {
		let parent = self.parent.as_deref().ok_or_else(|| {
			io::Error::new(io::ErrorKind::InvalidData, "trash item has no put-back location")
		})?;

		let name = self.name.as_deref().ok_or_else(|| {
			io::Error::new(io::ErrorKind::InvalidData, "trash item has no put-back name")
		})?;

		let mut components = Path::new(name).components();
		if !matches!(components.next(), Some(Component::Normal(_))) || components.next().is_some() {
			return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid trash put-back name"));
		}

		let top_path = Path::new("/").join(parent).join(name);
		Ok(if node.rel.as_os_str().is_empty() { top_path } else { top_path.join(&node.rel) })
	}
}
