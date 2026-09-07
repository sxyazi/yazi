use anyhow::bail;
use serde::Deserialize;
use strum::{EnumIs, EnumString, IntoStaticStr};

use crate::path::PathKind;

#[derive(
	Clone, Copy, Debug, Default, Deserialize, EnumIs, EnumString, Eq, Hash, IntoStaticStr, PartialEq,
)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum AuthKind {
	#[default]
	Regular,
	Mount,
	Hub,
	Scope,
	Sftp,
	View,
}

impl TryFrom<AuthKind> for PathKind {
	type Error = anyhow::Error;

	fn try_from(kind: AuthKind) -> Result<Self, Self::Error> {
		match kind {
			AuthKind::Regular | AuthKind::Mount | AuthKind::Hub => Ok(Self::Os),
			AuthKind::Scope | AuthKind::Sftp => Ok(Self::Unix),
			AuthKind::View => bail!("Auth kind has no path kind"),
		}
	}
}
