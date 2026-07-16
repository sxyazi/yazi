use serde::Deserialize;
use strum::{EnumString, IntoStaticStr};

#[derive(
	Clone, Copy, Debug, Default, Deserialize, EnumString, Eq, Hash, IntoStaticStr, PartialEq,
)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum AuthKind {
	#[default]
	Regular,
	Search,
	Mount,
	Hub,
	Scope,
	Sftp,
}

impl AuthKind {
	#[inline]
	pub fn is_local(self) -> bool {
		match self {
			Self::Regular | Self::Search => true,
			Self::Mount | Self::Hub | Self::Scope | Self::Sftp => false,
		}
	}

	#[inline]
	pub fn is_remote(self) -> bool {
		match self {
			Self::Regular | Self::Search | Self::Mount => false,
			Self::Hub | Self::Scope | Self::Sftp => true,
		}
	}

	#[inline]
	pub fn is_virtual(self) -> bool {
		match self {
			Self::Regular | Self::Search => false,
			Self::Mount | Self::Hub | Self::Scope | Self::Sftp => true,
		}
	}
}
