use std::{env, path::PathBuf};

use serde::{Deserialize, Deserializer, Serialize, de};

use crate::normalize_path;

#[derive(Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct ServiceSftp {
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

fn deserialize_path<'de, D>(deserializer: D) -> Result<PathBuf, D::Error>
where
	D: Deserializer<'de>,
{
	let mut path = PathBuf::deserialize(deserializer)?;
	if !path.as_os_str().is_empty() {
		path = normalize_path(path)
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
		normalize_path(path)
			.ok_or_else(|| de::Error::custom("identity_agent must be either empty or an absolute path"))
	}
}
