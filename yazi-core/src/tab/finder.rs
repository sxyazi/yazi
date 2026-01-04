use anyhow::Result;
use hashbrown::HashMap;
use yazi_fs::{Files, Filter, FilterCase};
use yazi_shared::{path::{AsPath, PathBufDyn}, url::UrlBuf};

use crate::tab::Folder;

pub struct Finder {
	pub filter:  Filter,
	pub matched: HashMap<PathBufDyn, u8>,
	lock:        FinderLock,
}

#[derive(Default)]
struct FinderLock {
	cwd:      UrlBuf,
	revision: u64,
}

impl Finder {
	pub fn new(s: &str, case: FilterCase) -> Result<Self> {
		Ok(Self {
			filter:  Filter::new(s, case)?,
			matched: Default::default(),
			lock:    Default::default(),
		})
	}

	pub fn prev(&self, files: &Files, cursor: usize, include: bool) -> Option<isize> {
		for i in !include as usize..files.len() {
			let idx = (cursor + files.len() - i) % files.len();
			if let Some(s) = files[idx].name()
				&& self.filter.matches(s)
			{
				return Some(idx as isize - cursor as isize);
			}
		}
		None
	}

	pub fn next(&self, files: &Files, cursor: usize, include: bool) -> Option<isize> {
		for i in !include as usize..files.len() {
			let idx = (cursor + i) % files.len();
			if let Some(s) = files[idx].name()
				&& self.filter.matches(s)
			{
				return Some(idx as isize - cursor as isize);
			}
		}
		None
	}

	pub fn catchup(&mut self, folder: &Folder) -> bool {
		if self.lock == *folder {
			return false;
		}
		self.matched.clear();

		let mut i = 0u8;
		for file in folder.files.iter() {
			if file.name().is_none_or(|s| !self.filter.matches(s)) {
				continue;
			}

			self.matched.insert(file.urn().into(), i);
			if self.matched.len() > 99 {
				break;
			}

			i += 1;
		}

		self.lock = folder.into();
		true
	}
}

impl Finder {
	pub fn matched_idx<T>(&self, folder: &Folder, urn: T) -> Option<u8>
	where
		T: AsPath,
	{
		if self.lock == *folder { self.matched.get(&urn.as_path()).copied() } else { None }
	}
}

// --- Lock
impl From<&Folder> for FinderLock {
	fn from(value: &Folder) -> Self {
		Self { cwd: value.url.clone(), revision: value.files.revision }
	}
}

impl PartialEq<Folder> for FinderLock {
	fn eq(&self, other: &Folder) -> bool {
		self.revision == other.files.revision && self.cwd == other.url
	}
}
