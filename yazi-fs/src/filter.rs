use std::{ffi::OsStr, fmt::Display, ops::Range};

use anyhow::Result;
use regex::bytes::{Regex, RegexBuilder};
use yazi_shared::event::Cmd;

pub struct Filter {
	raw:   String,
	regex: Regex,
}

impl Filter {
	pub fn new(s: &str, case: FilterCase) -> Result<Self> {
		let regex = match case {
			FilterCase::Smart => {
				let uppercase = s.chars().any(|c| c.is_uppercase());
				RegexBuilder::new(s).case_insensitive(!uppercase).build()?
			}
			FilterCase::Sensitive => Regex::new(s)?,
			FilterCase::Insensitive => RegexBuilder::new(s).case_insensitive(true).build()?,
		};
		Ok(Self { raw: s.to_owned(), regex })
	}

	#[inline]
	pub fn matches(&self, name: &OsStr) -> bool { self.regex.is_match(name.as_encoded_bytes()) }

	#[inline]
	pub fn highlighted(&self, name: &OsStr) -> Option<Vec<Range<usize>>> {
		self.regex.find(name.as_encoded_bytes()).map(|m| vec![m.range()])
	}
}

impl PartialEq for Filter {
	fn eq(&self, other: &Self) -> bool { self.raw == other.raw }
}

impl Display for Filter {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_str(&self.raw) }
}

#[derive(Default, PartialEq, Eq)]
pub enum FilterCase {
	Smart,
	#[default]
	Sensitive,
	Insensitive,
}

impl From<&Cmd> for FilterCase {
	fn from(c: &Cmd) -> Self {
		match (c.bool("smart"), c.bool("insensitive")) {
			(true, _) => Self::Smart,
			(_, false) => Self::Sensitive,
			(_, true) => Self::Insensitive,
		}
	}
}
