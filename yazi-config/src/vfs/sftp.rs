use std::{env, ops::Deref, path::PathBuf, sync::Arc};

use serde::{Deserialize, Deserializer, Serialize, de};
use yazi_fs::path::sanitize_path;
use yazi_shared::auth::Auth;

#[derive(Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct ServiceSftp {
	#[serde(skip, default)]
	pub auth:           Arc<Auth>,
	pub host:           String,
	pub user:           String,
	pub port:           u16,
	pub password:       Option<String>,
	#[serde(default, deserialize_with = "deserialize_path")]
	pub key_file:       PathBuf,
	pub key_passphrase: Option<String>,
	#[serde(default, deserialize_with = "deserialize_path")]
	pub cert_file:      PathBuf,
	#[serde(default)]
	pub no_cert_verify: bool,
	#[serde(default = "default_identity_agent", deserialize_with = "deserialize_identity_agent")]
	pub identity_agent: PathBuf,
}

impl Deref for ServiceSftp {
	type Target = Auth;

	fn deref(&self) -> &Self::Target { &self.auth }
}

fn deserialize_path<'de, D>(deserializer: D) -> Result<PathBuf, D::Error>
where
	D: Deserializer<'de>,
{
	let mut path = PathBuf::deserialize(deserializer)?;
	if !path.as_os_str().is_empty() {
		path = sanitize_path(path)
			.ok_or_else(|| de::Error::custom("path must be either empty or an absolute path"))?;
	}

	Ok(path)
}

fn default_identity_agent() -> PathBuf {
	env::var_os("SSH_AUTH_SOCK").map(PathBuf::from).filter(|p| p.is_absolute()).unwrap_or_default()
}

fn deserialize_identity_agent<'de, D>(deserializer: D) -> Result<PathBuf, D::Error>
where
	D: Deserializer<'de>,
{
	let path = PathBuf::deserialize(deserializer)?;
	if path.as_os_str().is_empty() {
		Ok(default_identity_agent())
	} else {
		sanitize_path(path)
			.ok_or_else(|| de::Error::custom("identity_agent must be either empty or an absolute path"))
	}
}
