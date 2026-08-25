use std::{fmt, str::FromStr};

use anyhow::{Result, bail};
use serde_with::DeserializeFromStr;

use crate::KebabCasedKey;

#[derive(Clone, Debug, DeserializeFromStr, Eq, Hash, PartialEq)]
pub enum Scheme {
	Regular,
	Search,
	Sftp,
	Custom(KebabCasedKey),
}

impl AsRef<str> for Scheme {
	fn as_ref(&self) -> &str { self.as_str() }
}

impl AsRef<[u8]> for Scheme {
	fn as_ref(&self) -> &[u8] { self.as_str().as_bytes() }
}

impl PartialEq<str> for Scheme {
	fn eq(&self, other: &str) -> bool { self.as_str() == other }
}

impl PartialEq<&str> for Scheme {
	fn eq(&self, other: &&str) -> bool { self == *other }
}

impl PartialEq<&String> for Scheme {
	fn eq(&self, other: &&String) -> bool { self == other.as_str() }
}

impl PartialEq<&Self> for Scheme {
	fn eq(&self, other: &&Self) -> bool { self == *other }
}

impl fmt::Display for Scheme {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { self.as_str().fmt(f) }
}

impl FromStr for Scheme {
	type Err = anyhow::Error;

	fn from_str(s: &str) -> Result<Self> {
		Ok(match s {
			"regular" => Self::Regular,
			"search" => Self::Search,
			"sftp" => Self::Sftp,
			_ if let Some(s) = KebabCasedKey::new(s) => Self::Custom(s),
			_ => bail!("scheme must be 1-20 characters in kebab-case, got: {s}"),
		})
	}
}

impl Scheme {
	pub(crate) fn as_str(&self) -> &str {
		match self {
			Self::Regular => "regular",
			Self::Search => "search",
			Self::Sftp => "sftp",
			Self::Custom(s) => s,
		}
	}
}
