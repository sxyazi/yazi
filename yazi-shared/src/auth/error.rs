use std::fmt::{self, Display, Formatter};

use anyhow::anyhow;

use super::Scheme;
use crate::domain::Domain;

#[derive(Debug)]
pub(crate) struct AuthError<'a> {
	pub(crate) scheme: &'a Scheme,
	pub(crate) domain: &'a Domain<'a>,
}

impl Display for AuthError<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		write!(f, "unknown authority: {}://{}", self.scheme, self.domain)
	}
}

impl From<AuthError<'_>> for anyhow::Error {
	fn from(err: AuthError<'_>) -> Self { anyhow!("{err}") }
}

impl From<AuthError<'_>> for mlua::Error {
	fn from(err: AuthError<'_>) -> Self { Self::runtime(err) }
}
