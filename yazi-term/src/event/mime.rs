use std::str::SplitWhitespace;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MimeList(String);

impl MimeList {
	pub fn new(b: Vec<u8>) -> Option<Self> { Some(Self(String::from_utf8(b).ok()?)) }

	pub fn iter(&self) -> SplitWhitespace<'_> { self.0.split_whitespace() }
}
