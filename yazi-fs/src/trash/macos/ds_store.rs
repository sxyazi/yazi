use std::{ffi::{OsStr, OsString}, fs, io::{self, Write}, path::{Path, PathBuf}};

use ds_parser::{FieldKey, Record, Value};
use hashbrown::HashMap;
use yazi_shim::{OsStrExt2, path::OsPathExt};

#[derive(Default)]
pub(super) struct DsStore {
	parent: Option<PathBuf>,
	name:   Option<OsString>,
}

impl DsStore {
	fn open(path: &Path) -> io::Result<ds_parser::DSStore> {
		match fs::read(path) {
			Ok(bytes) => ds_parser::parse(&bytes).map_err(io::Error::other),
			Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(ds_parser::DSStore::new(vec![])),
			Err(e) => Err(e),
		}
	}

	pub(super) fn set(root: &Path, name: &str, original: &Path) -> io::Result<()> {
		if original.non_empty().is_some_and(|p| !p.is_absolute() || p.file_name().is_none()) {
			return Err(io::Error::new(io::ErrorKind::InvalidInput, "invalid trash original path"));
		}

		// Keep the lock file in place so concurrent writers lock the same inode.
		let lock = fs::OpenOptions::new()
			.create(true)
			.truncate(false)
			.write(true)
			.open(root.join(".DS_Store.yazi-lock"))?;
		lock.lock()?;

		// Open the store.
		let path = root.join(".DS_Store");
		let mut store = Self::open(&path)?;

		// Remove the original path if it exists.
		let removed = Self::remove_original(&mut store, name);
		if !removed && original.non_empty().is_none() {
			return Ok(());
		}

		Self::add_original(&mut store, name, original)?;
		let bytes = store.to_bytes().map_err(io::Error::other)?;
		let tmp = root.join(format!(".DS_Store.yazi-{:016x}", rand::random::<u64>()));

		let mut file = fs::File::create_new(&tmp)?;
		file.write_all(&bytes)?;
		fs::rename(&tmp, path)
	}

	pub(super) fn parse(path: &Path) -> io::Result<HashMap<OsString, Self>> {
		let mut locations = HashMap::<OsString, Self>::new();
		for record in Self::open(path)?.records {
			let Value::Ustr(value) = record.value else { continue };
			if value.is_empty() {
				continue;
			}

			let location = locations.entry_ref(OsStr::new(&record.name)).or_default();
			match record.field {
				FieldKey::ptbL => location.parent = Some(value.into()),
				FieldKey::ptbN => location.name = Some(value.into()),
				_ => {}
			}
		}

		Ok(locations)
	}

	pub(super) fn join(&self, rel: &Path) -> io::Result<PathBuf> {
		if !rel.is_joinable() || rel.has_parent_component() {
			return Err(io::Error::new(io::ErrorKind::InvalidInput, "invalid trash entry path"));
		}

		let parent = self.parent.as_deref().ok_or_else(|| {
			io::Error::new(io::ErrorKind::InvalidData, "trash item has no put-back location")
		})?;

		let name = self.name.as_deref().ok_or_else(|| {
			io::Error::new(io::ErrorKind::InvalidData, "trash item has no put-back name")
		})?;

		if !name.is_filename() {
			return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid trash put-back name"));
		}

		let top_path = Path::new("/").join(parent).join(name);
		Ok(if rel.as_os_str().is_empty() { top_path } else { top_path.join(rel) })
	}

	pub(super) fn is_internal(name: &OsStr) -> bool {
		match name.as_encoded_bytes() {
			b".DS_Store" | b".DS_Store.yazi-lock" => true,
			name => name
				.strip_prefix(b".DS_Store.yazi-")
				.is_some_and(|s| s.len() == 16 && s.iter().all(u8::is_ascii_hexdigit)),
		}
	}

	fn add_original(store: &mut ds_parser::DSStore, name: &str, original: &Path) -> io::Result<()> {
		let Some((parent, original_name)) = original.pair() else { return Ok(()) };

		for (field, value) in [(FieldKey::ptbL, parent.as_os_str()), (FieldKey::ptbN, original_name)] {
			let value = value.to_str().ok_or_else(|| {
				io::Error::new(io::ErrorKind::InvalidInput, "invalid trash original path")
			})?;
			store.records.push(Record::new(name, field, value));
		}
		Ok(())
	}

	fn remove_original(store: &mut ds_parser::DSStore, name: &str) -> bool {
		let len = store.records.len();
		store.records.retain(|r| r.name != name || !matches!(r.field, FieldKey::ptbL | FieldKey::ptbN));
		store.records.len() != len
	}
}
