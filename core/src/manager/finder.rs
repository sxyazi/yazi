use std::{collections::BTreeMap, ffi::OsStr};

use anyhow::Result;
use regex::bytes::Regex;
use shared::Url;

use crate::files::Files;

pub struct Finder {
	query:   Regex,
	matched: BTreeMap<Url, u8>,
	version: u64,
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
				.find(|(_, f)| f.name().map_or(false, |n| self.matches(n)))
				.map(|(i, _)| -(i as isize) - 1)
		} else {
			files
				.iter()
				.skip(cursor + 1)
				.enumerate()
				.find(|(_, f)| f.name().map_or(false, |n| self.matches(n)))
				.map(|(i, _)| i as isize + 1)
		}
	}

	pub(super) fn ring(&self, files: &Files, cursor: usize, prev: bool) -> Option<isize> {
		if prev {
			files
				.iter()
				.take(cursor + 1)
				.rev()
				.enumerate()
				.find(|(_, f)| f.name().map_or(false, |n| self.matches(n)))
				.map(|(i, _)| -(i as isize))
				.or_else(|| {
					files
						.iter()
						.skip(cursor + 1)
						.enumerate()
						.find(|(_, f)| f.name().map_or(false, |n| self.matches(n)))
						.map(|(i, _)| i as isize + 1)
				})
		} else {
			files
				.iter()
				.skip(cursor)
				.enumerate()
				.find(|(_, f)| f.name().map_or(false, |n| self.matches(n)))
				.map(|(i, _)| i as isize)
				.or_else(|| {
					files
						.iter()
						.take(cursor)
						.rev()
						.enumerate()
						.find(|(_, f)| f.name().map_or(false, |n| self.matches(n)))
						.map(|(i, _)| -(i as isize) - 1)
				})
		}
	}

	pub(super) fn catchup(&mut self, files: &Files) -> bool {
		if self.version == files.version() {
			return false;
		}
		self.matched.clear();

		let mut i = 0u8;
		for file in files.iter() {
			if file.name().map(|n| self.matches(n)) != Some(true) {
				continue;
			}

			self.matched.insert(file.url_owned(), i);
			if self.matched.len() > 99 {
				break;
			}

			i += 1;
		}

		self.version = files.version();
		true
	}

	#[inline]
	fn matches(&self, name: &OsStr) -> bool {
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
	pub fn matched(&self) -> &BTreeMap<Url, u8> { &self.matched }

	#[inline]
	pub fn has_matched(&self) -> bool { !self.matched.is_empty() }

	#[inline]
	pub fn matched_idx(&self, url: &Url) -> Option<u8> {
		if let Some((_, &idx)) = self.matched.iter().find(|(u, _)| *u == url) {
			return Some(idx);
		}
		if url.file_name().map(|n| self.matches(n)) == Some(true) {
			return Some(100);
		}
		None
	}
}
