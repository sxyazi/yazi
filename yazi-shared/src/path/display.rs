use crate::path::PathDyn;

pub struct Display<'a>(pub PathDyn<'a>);

impl std::fmt::Display for Display<'_> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self.0 {
			PathDyn::Os(p) => write!(f, "{}", p.display()),
			PathDyn::Unix(p) => write!(f, "{}", p.display()),
		}
	}
}
