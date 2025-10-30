use std::{ffi::OsStr, fmt::Display, ops::Range};

use anyhow::Result;
use regex::bytes::{Regex, RegexBuilder};
use yazi_shared::{event::Cmd, url::Urn};

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
	#[allow(private_bounds)]
	pub fn matches(&self, name: impl Needle) -> bool { self.regex.is_match(name.needle()) }

	#[inline]
	pub fn highlighted(&self, name: impl AsRef<OsStr>) -> Option<Vec<Range<usize>>> {
		self.regex.find(name.as_ref().as_encoded_bytes()).map(|m| vec![m.range()])
	}
}

impl PartialEq for Filter {
	fn eq(&self, other: &Self) -> bool { self.raw == other.raw }
}

impl Display for Filter {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_str(&self.raw) }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
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

// --- Needle
trait Needle {
	fn needle(&self) -> &[u8];
}

impl Needle for &OsStr {
	fn needle(&self) -> &[u8] { self.as_encoded_bytes() }
}

impl Needle for &Urn {
	fn needle(&self) -> &[u8] { self.encoded_bytes() }
}
