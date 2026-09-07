use std::{fmt::{self, Display}, ops::Deref};

use crate::{auth::{AuthKind, EncodePrefix}, spec::Spec, url::Url};

// --- Encode
#[derive(Clone, Copy)]
pub struct Encode<'a>(pub Url<'a>);

impl<'a> Deref for Encode<'a> {
	type Target = Url<'a>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl Display for Encode<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let auth = self.auth();
		write!(f, "{}{}{}", auth.encode(false), EncodePorts(self.0), EncodePrefix(auth))?;

		if self.0.is_view() {
			let source = self.base().physical();
			if !source.is_regular() {
				Self(source).fmt(f)?;
			}
		}

		Ok(())
	}
}

// --- EncodePorts
#[derive(Clone, Copy)]
pub struct EncodePorts<'a>(pub(crate) Url<'a>);

impl<'a> Deref for EncodePorts<'a> {
	type Target = Url<'a>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl Display for EncodePorts<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let (uri, urn) = Spec::retrieve_ports(self.0);
		let default = match self.kind() {
			AuthKind::Regular => return Ok(()),
			AuthKind::View | AuthKind::Mount => 0,
			_ => self.name().is_some() as usize,
		};

		match (uri != default, urn != default) {
			(true, true) => write!(f, ":{uri}:{urn}"),
			(true, false) => write!(f, ":{uri}"),
			(false, true) => write!(f, "::{urn}"),
			(false, false) => Ok(()),
		}
	}
}
