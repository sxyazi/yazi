use std::{io, path::PathBuf};

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

impl Provider {
	pub(super) fn reshape(self) -> io::Result<Self> {
		match self {
			Self::Sftp(p) => p.reshape().map(Self::Sftp),
		}
	}
}

// --- SFTP
#[derive(Deserialize, Hash, Serialize, Eq, PartialEq)]
pub struct ProviderSftp {
	pub host:           String,
	pub user:           String,
	pub port:           u16,
	pub password:       Option<String>,
	pub key_file:       Option<PathBuf>,
	pub key_passphrase: Option<String>,
	// FIXME: set default: $SSH_AUTH_SOCK
	pub identity_agent: Option<PathBuf>,
}

impl ProviderSftp {
	fn reshape(self) -> io::Result<Self> {
		// FIXME: expand the path
		// if let Some(key_file) = self.key_file {}
		// if let Some(identity_agent) = self.identity_agent {}

		Ok(self)
	}
}
