use crate::{scheme::Encode, url::Url};

pub struct Display<'a>(pub Url<'a>);

impl std::fmt::Display for Display<'_> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let (kind, loc) = (self.0.kind(), self.0.loc());
		if kind.is_virtual() {
			Encode(self.0).fmt(f)?;
		}
		loc.display().fmt(f)
	}
}
