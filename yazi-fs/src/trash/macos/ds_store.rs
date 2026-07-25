use std::{ffi::{OsStr, OsString}, fs, io, path::{Component, Path, PathBuf}};

use ds_parser::Value;
use hashbrown::HashMap;
use yazi_shim::path::PathExt;

#[derive(Default)]
pub(super) struct DsStore {
	parent: Option<PathBuf>,
	name:   Option<OsString>,
}

impl DsStore {
	pub(super) fn parse(path: &Path) -> io::Result<HashMap<OsString, DsStore>> {
		let bytes = fs::read(path)?;
		let store = ds_parser::parse(&bytes).map_err(io::Error::other)?;

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

	pub(super) fn join(&self, rel: &Path) -> io::Result<PathBuf> {
		if !rel.is_relative() || rel.has_parent_component() {
			return Err(io::Error::new(io::ErrorKind::InvalidInput, "invalid trash entry path"));
		}

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
		Ok(if rel.as_os_str().is_empty() { top_path } else { top_path.join(rel) })
	}
}
