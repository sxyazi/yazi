use std::ops::Deref;

use crate::path::PathDyn;

pub struct Display<'a>(pub(crate) PathDyn<'a>);

impl<'a> Deref for Display<'a> {
	type Target = PathDyn<'a>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl std::fmt::Display for Display<'_> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self.0 {
			PathDyn::Os(p) => write!(f, "{}", p.display()),
			PathDyn::Unix(p) => write!(f, "{}", p.display()),
		}
	}
}
