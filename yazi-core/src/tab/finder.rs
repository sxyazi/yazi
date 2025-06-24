use std::collections::HashMap;

use anyhow::Result;
use yazi_fs::{Files, Filter, FilterCase};
use yazi_shared::url::{Url, Urn, UrnBuf};

use crate::tab::Folder;

pub struct Finder {
	pub filter:  Filter,
	pub matched: HashMap<UrnBuf, u8>,
	lock:        FinderLock,
}

#[derive(Default)]
struct FinderLock {
	cwd:      Url,
	revision: u64,
}

impl Finder {
	pub(super) fn new(s: &str, case: FilterCase) -> Result<Self> {
		Ok(Self {
			filter:  Filter::new(s, case)?,
			matched: Default::default(),
			lock:    Default::default(),
		})
	}

	pub(super) fn prev(&self, files: &Files, cursor: usize, include: bool) -> Option<isize> {
		for i in !include as usize..files.len() {
			let idx = (cursor + files.len() - i) % files.len();
			if self.filter.matches(files[idx].name()) {
				return Some(idx as isize - cursor as isize);
			}
		}
		None
	}

	pub(super) fn next(&self, files: &Files, cursor: usize, include: bool) -> Option<isize> {
		for i in !include as usize..files.len() {
			let idx = (cursor + i) % files.len();
			if self.filter.matches(files[idx].name()) {
				return Some(idx as isize - cursor as isize);
			}
		}
		None
	}

	pub(super) fn catchup(&mut self, folder: &Folder) -> bool {
		if self.lock == *folder {
			return false;
		}
		self.matched.clear();

		let mut i = 0u8;
		for file in folder.files.iter() {
			if !self.filter.matches(file.name()) {
				continue;
			}

			self.matched.insert(file.urn_owned(), i);
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
	#[inline]
	pub fn matched_idx(&self, folder: &Folder, urn: &Urn) -> Option<u8> {
		if self.lock == *folder { self.matched.get(urn).copied() } else { None }
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
