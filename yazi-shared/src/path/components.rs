use std::iter::FusedIterator;

use typed_path::Components as _;

use crate::{path::{Component, PathDyn}, strand::Strand};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Components<'a> {
	Os(std::path::Components<'a>),
	Unix(typed_path::UnixComponents<'a>),
}

impl<'a> From<&'a std::path::Path> for Components<'a> {
	fn from(value: &'a std::path::Path) -> Self { Self::Os(value.components()) }
}

impl<'a> From<&'a typed_path::UnixPath> for Components<'a> {
	fn from(value: &'a typed_path::UnixPath) -> Self { Self::Unix(value.components()) }
}

impl<'a> Components<'a> {
	pub fn auth_depth(self) -> usize { self.filter(Component::has_auth).count() }

	pub fn path(&self) -> PathDyn<'a> {
		match self {
			Self::Os(c) => PathDyn::Os(c.as_path()),
			Self::Unix(c) => PathDyn::Unix(c.as_path()),
		}
	}

	pub fn strand(&self) -> Strand<'a> {
		match self {
			Self::Os(c) => Strand::Os(c.as_path().as_os_str()),
			Self::Unix(c) => Strand::Bytes(c.as_bytes()),
		}
	}
}

impl<'a> Iterator for Components<'a> {
	type Item = Component<'a>;

	fn next(&mut self) -> Option<Component<'a>> {
		match self {
			Self::Os(c) => c.next().map(Into::into),
			Self::Unix(c) => c.next().map(Into::into),
		}
	}
}

impl<'a> DoubleEndedIterator for Components<'a> {
	fn next_back(&mut self) -> Option<Component<'a>> {
		match self {
			Self::Os(c) => c.next_back().map(Into::into),
			Self::Unix(c) => c.next_back().map(Into::into),
		}
	}
}

impl FusedIterator for Components<'_> {}
