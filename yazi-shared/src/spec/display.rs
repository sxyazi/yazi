use std::{fmt, ops::Deref};

use super::EncodePorts;
use crate::{auth::DisplayPrefix, url::Url};

pub struct Display<'a>(pub(crate) Url<'a>);

impl<'a> Deref for Display<'a> {
	type Target = Url<'a>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl fmt::Display for Display<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let auth = self.auth();
		write!(f, "{auth}{}{}", EncodePorts(self.0), DisplayPrefix(auth))?;

		if self.is_view() {
			Self(self.0.base().physical()).fmt(f)?;
		}
		Ok(())
	}
}
