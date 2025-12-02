use std::{io, path::PathBuf};

use serde::{Deserialize, Serialize};
use yazi_fs::path::expand_url;

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
	pub(super) fn reshape(&mut self) -> io::Result<()> {
		match self {
			Self::Sftp(p) => p.reshape(),
		}
	}
}

// --- SFTP
#[derive(Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct ProviderSftp {
	pub host:           String,
	pub user:           String,
	pub port:           u16,
	pub password:       Option<String>,
	#[serde(default)]
	pub key_file:       PathBuf,
	pub key_passphrase: Option<String>,
	#[serde(default)]
	pub identity_agent: PathBuf,
}

impl ProviderSftp {
	fn reshape(&mut self) -> io::Result<()> {
		if !self.key_file.as_os_str().is_empty() {
			self.key_file = expand_url(&self.key_file)
				.into_local()
				.ok_or_else(|| io::Error::other("key_file must be a path within local filesystem"))?;
		}

		self.identity_agent = if self.identity_agent.as_os_str().is_empty() {
			std::env::var_os("SSH_AUTH_SOCK")
				.map(PathBuf::from)
				.filter(|p| p.is_absolute())
				.unwrap_or_default()
		} else {
			expand_url(&self.identity_agent)
				.into_local()
				.ok_or_else(|| io::Error::other("identity_agent must be a path within local filesystem"))?
		};

		Ok(())
	}
}
