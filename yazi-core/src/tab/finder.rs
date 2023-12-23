use std::{collections::BTreeMap, ffi::OsStr, ops::Range};

use anyhow::Result;
use regex::bytes::{Regex, RegexBuilder};
use yazi_shared::fs::Url;

use crate::folder::Files;

#[derive(PartialEq, Eq)]
pub enum FinderCase {
	Smart,
	Sensitive,
	Insensitive,
}

pub struct Finder {
	query:   Regex,
	matched: BTreeMap<Url, u8>,
	version: u64,
}

impl Finder {
	pub(super) fn new(s: &str, case: FinderCase) -> Result<Self> {
		let query = match case {
			FinderCase::Smart => {
				let uppercase = s.chars().any(|c| c.is_uppercase());
				RegexBuilder::new(s).case_insensitive(!uppercase).build()?
			}
			FinderCase::Sensitive => Regex::new(s)?,
			FinderCase::Insensitive => RegexBuilder::new(s).case_insensitive(true).build()?,
		};
		Ok(Self { query, matched: Default::default(), version: 0 })
	}

	pub(super) fn prev(&self, files: &Files, cursor: usize, include: bool) -> Option<isize> {
		for i in !include as usize..files.len() {
			let idx = (cursor + files.len() - i) % files.len();
			if files[idx].name().is_some_and(|n| self.matches(n)) {
				return Some(idx as isize - cursor as isize);
			}
		}
		None
	}

	pub(super) fn next(&self, files: &Files, cursor: usize, include: bool) -> Option<isize> {
		for i in !include as usize..files.len() {
			let idx = (cursor + i) % files.len();
			if files[idx].name().is_some_and(|n| self.matches(n)) {
				return Some(idx as isize - cursor as isize);
			}
		}
		None
	}

	pub(super) fn catchup(&mut self, files: &Files) -> bool {
		if self.version == files.revision {
			return false;
		}
		self.matched.clear();

		let mut i = 0u8;
		for file in files.iter() {
			if file.name().map(|n| self.matches(n)) != Some(true) {
				continue;
			}

			self.matched.insert(file.url(), i);
			if self.matched.len() > 99 {
				break;
			}

			i += 1;
		}

		self.version = files.revision;
		true
	}

	#[inline]
	fn matches(&self, name: &OsStr) -> bool { self.query.is_match(name.as_encoded_bytes()) }

	/// Explode the name into three parts: head, body, tail.
	#[inline]
	pub fn highlighted(&self, name: &OsStr) -> Option<Vec<Range<usize>>> {
		self.query.find(name.as_encoded_bytes()).map(|m| vec![m.range()])
	}
}

impl Finder {
	#[inline]
	pub fn matched(&self) -> &BTreeMap<Url, u8> { &self.matched }

	#[inline]
	pub fn matched_idx(&self, url: &Url) -> Option<u8> {
		if let Some((_, &idx)) = self.matched.iter().find(|(u, _)| *u == url) {
			return Some(idx);
		}
		None
	}
}
