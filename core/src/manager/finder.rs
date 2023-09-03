use std::{collections::BTreeMap, ffi::OsStr, path::{Path, PathBuf}};

use anyhow::Result;
use regex::bytes::Regex;

use crate::files::Files;

pub struct Finder {
	query:   Regex,
	matched: BTreeMap<PathBuf, u8>,
	version: usize,
}

impl Finder {
	pub(super) fn new(s: &str) -> Result<Self> {
		Ok(Self { query: Regex::new(s)?, matched: Default::default(), version: 0 })
	}

	pub(super) fn arrow(&self, files: &Files, cursor: usize, prev: bool) -> Option<isize> {
		if prev {
			files
				.iter()
				.take(cursor)
				.rev()
				.enumerate()
				.find(|(_, f)| f.name().map_or(false, |n| self.is_match(n)))
				.map(|(i, _)| -(i as isize) - 1)
		} else {
			files
				.iter()
				.skip(cursor + 1)
				.enumerate()
				.find(|(_, f)| f.name().map_or(false, |n| self.is_match(n)))
				.map(|(i, _)| i as isize + 1)
		}
	}

	pub(super) fn catchup(&mut self, files: &Files) -> bool {
		if self.version == files.version() {
			return false;
		}

		self.matched.clear();

		let mut i = 0u8;
		for file in files.iter() {
			if file.name().map(|n| self.is_match(n)) != Some(true) {
				continue;
			}

			self.matched.insert(file.path_owned(), i);
			if self.matched.len() > 99 {
				break;
			}

			i += 1;
		}

		self.version = files.version();
		true
	}

	#[inline]
	fn is_match(&self, name: &OsStr) -> bool {
		#[cfg(target_os = "windows")]
		{
			self.query.is_match(name.to_string_lossy().as_bytes())
		}
		#[cfg(not(target_os = "windows"))]
		{
			use std::os::unix::ffi::OsStrExt;
			self.query.is_match(name.as_bytes())
		}
	}
}

impl Finder {
	#[inline]
	pub fn matched(&self) -> &BTreeMap<PathBuf, u8> { &self.matched }

	#[inline]
	pub fn matched_idx(&self, path: &Path) -> Option<u8> {
		if let Some((_, &idx)) = self.matched.iter().find(|(p, _)| *p == path) {
			return Some(idx);
		}
		if path.file_name().map(|n| self.is_match(n)) == Some(true) {
			return Some(100);
		}
		None
	}
}
