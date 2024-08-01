use std::collections::HashMap;

use anyhow::Result;
use yazi_fs::{Files, Filter, FilterCase};
use yazi_shared::fs::Url;

pub struct Finder {
	pub filter: Filter,
	matched:    HashMap<Url, u8>,
	revision:   u64,
}

impl Finder {
	pub(super) fn new(s: &str, case: FilterCase) -> Result<Self> {
		Ok(Self { filter: Filter::new(s, case)?, matched: Default::default(), revision: 0 })
	}

	pub(super) fn prev(&self, files: &Files, cursor: usize, include: bool) -> Option<isize> {
		for i in !include as usize..files.len() {
			let idx = (cursor + files.len() - i) % files.len();
			if files[idx].name().is_some_and(|n| self.filter.matches(n)) {
				return Some(idx as isize - cursor as isize);
			}
		}
		None
	}

	pub(super) fn next(&self, files: &Files, cursor: usize, include: bool) -> Option<isize> {
		for i in !include as usize..files.len() {
			let idx = (cursor + i) % files.len();
			if files[idx].name().is_some_and(|n| self.filter.matches(n)) {
				return Some(idx as isize - cursor as isize);
			}
		}
		None
	}

	pub(super) fn catchup(&mut self, files: &Files) -> bool {
		if self.revision == files.revision {
			return false;
		}
		self.matched.clear();

		let mut i = 0u8;
		for file in files.iter() {
			if file.name().map(|n| self.filter.matches(n)) != Some(true) {
				continue;
			}

			self.matched.insert(file.url(), i);
			if self.matched.len() > 99 {
				break;
			}

			i += 1;
		}

		self.revision = files.revision;
		true
	}
}

impl Finder {
	#[inline]
	pub fn matched(&self) -> &HashMap<Url, u8> { &self.matched }

	#[inline]
	pub fn matched_idx(&self, url: &Url) -> Option<u8> { self.matched.get(url).copied() }
}
