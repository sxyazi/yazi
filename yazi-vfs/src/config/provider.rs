use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum Provider {
	Sftp(ProviderSftp),
}

impl TryFrom<&'static Provider> for &'static ProviderSftp {
	type Error = &'static str;

	fn try_from(value: &'static Provider) -> Result<Self, Self::Error> {
		match value {
			Provider::Sftp(p) => Ok(p),
		}
	}
}

// --- SFTP
#[derive(Deserialize, Hash, Serialize, Eq, PartialEq)]
pub struct ProviderSftp {
	pub host:     String,
	pub user:     String,
	pub port:     u16,
	pub password: Option<String>,
	pub key_file: Option<PathBuf>,
}
