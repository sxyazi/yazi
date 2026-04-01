use std::{io, mem, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::normalize_path;

#[derive(Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum Service {
	S3(ServiceS3),
	Sftp(ServiceSftp),
}

impl TryFrom<&'static Service> for &'static ServiceS3 {
	type Error = &'static str;

	fn try_from(value: &'static Service) -> Result<Self, Self::Error> {
		match value {
			Service::S3(p) => Ok(p),
			_ => Err("expected s3 service"),
		}
	}
}

impl TryFrom<&'static Service> for &'static ServiceSftp {
	type Error = &'static str;

	fn try_from(value: &'static Service) -> Result<Self, Self::Error> {
		match value {
			Service::Sftp(p) => Ok(p),
			_ => Err("expected sftp service"),
		}
	}
}

impl Service {
	pub(super) fn reshape(&mut self) -> io::Result<()> {
		match self {
			Self::S3(p) => p.reshape(),
			Self::Sftp(p) => p.reshape(),
		}
	}
}

// --- S3
#[derive(Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct ServiceS3 {
	pub region:            Option<String>,
	pub endpoint:          Option<String>,
	pub access_key_id:     Option<String>,
	pub secret_access_key: Option<String>,
	pub session_token:     Option<String>,
	#[serde(default)]
	pub force_path_style:  bool,
	#[serde(default)]
	pub allow_http:        bool,
}

impl ServiceS3 {
	fn reshape(&mut self) -> io::Result<()> {
		self.region = trim_option(self.region.take());
		self.endpoint = self.endpoint.take().and_then(|s| {
			let s = s.trim().trim_end_matches('/').to_owned();
			(!s.is_empty()).then_some(s)
		});
		self.access_key_id = trim_option(self.access_key_id.take());
		self.secret_access_key = trim_option(self.secret_access_key.take());
		self.session_token = trim_option(self.session_token.take());

		Ok(())
	}
}

fn trim_option(value: Option<String>) -> Option<String> {
	value.and_then(|s| {
		let s = s.trim().to_owned();
		(!s.is_empty()).then_some(s)
	})
}

// --- SFTP
#[derive(Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct ServiceSftp {
	pub host:           String,
	pub user:           String,
	pub port:           u16,
	pub password:       Option<String>,
	#[serde(default)]
	pub key_file:       PathBuf,
	pub key_passphrase: Option<String>,
	#[serde(default)]
	pub cert_file:      PathBuf,
	#[serde(default)]
	pub no_cert_verify: bool,
	#[serde(default)]
	pub identity_agent: PathBuf,
}

impl ServiceSftp {
	fn reshape(&mut self) -> io::Result<()> {
		if !self.key_file.as_os_str().is_empty() {
			self.key_file = normalize_path(mem::take(&mut self.key_file))
				.ok_or_else(|| io::Error::other("key_file must be either empty or an absolute path"))?;
		}

		if !self.cert_file.as_os_str().is_empty() {
			self.cert_file = normalize_path(mem::take(&mut self.cert_file))
				.ok_or_else(|| io::Error::other("cert_file must be either empty or an absolute path"))?;
		}

		self.identity_agent = if self.identity_agent.as_os_str().is_empty() {
			std::env::var_os("SSH_AUTH_SOCK")
				.map(PathBuf::from)
				.filter(|p| p.is_absolute())
				.unwrap_or_default()
		} else {
			normalize_path(mem::take(&mut self.identity_agent)).ok_or_else(|| {
				io::Error::other("identity_agent must be either empty or an absolute path")
			})?
		};

		Ok(())
	}
}
