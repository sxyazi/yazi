use std::{ffi::OsStr, ops::Range};

use anyhow::Result;
use regex::bytes::{Regex, RegexBuilder};
use yazi_shared::event::Exec;

pub struct Filter {
	raw:   String,
	regex: Regex,
}

impl PartialEq for Filter {
	fn eq(&self, other: &Self) -> bool { self.raw == other.raw }
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

#[derive(Default, PartialEq, Eq)]
pub enum FilterCase {
	Smart,
	#[default]
	Sensitive,
	Insensitive,
}

impl From<&Exec> for FilterCase {
	fn from(e: &Exec) -> Self {
		match (e.named.contains_key("smart"), e.named.contains_key("insensitive")) {
			(true, _) => Self::Smart,
			(_, false) => Self::Sensitive,
			(_, true) => Self::Insensitive,
		}
	}
}
