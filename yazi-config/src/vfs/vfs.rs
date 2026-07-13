use std::io;

use anyhow::{Context, Result};
use serde::{Deserialize, Deserializer};
use yazi_codegen::DeserializeOver;
use yazi_fs::{Xdg, ok_or_not_found};
use yazi_shared::auth::{Auth, AuthInventory};
use yazi_shim::toml::DeserializeOverWith;

use super::{Authorities, Service};
use crate::VFS;

#[derive(Deserialize, DeserializeOver)]
pub struct Vfs {
	#[serde(flatten)]
	pub authorities: Authorities,
}

impl Vfs {
	pub fn service<P>(auth: &Auth) -> io::Result<P>
	where
		P: TryFrom<&'static Service, Error = &'static str>,
	{
		let Some(value) = VFS.authorities.get(&auth.scheme, &auth.domain) else {
			return Err(io::Error::other(format!("No such VFS service: {auth}")));
		};

		match value.try_into() {
			Ok(p) => Ok(p),
			Err(e) => Err(io::Error::other(format!("VFS service `{auth}` has wrong kind: {e}"))),
		}
	}

	pub(crate) fn read() -> Result<String> {
		let p = Xdg::config_dir().join("vfs.toml");
		ok_or_not_found(std::fs::read_to_string(&p))
			.with_context(|| format!("Failed to read config {p:?}"))
	}
}

impl DeserializeOverWith for Vfs {
	fn deserialize_over_with<'de, D: Deserializer<'de>>(self, de: D) -> Result<Self, D::Error> {
		Ok(Self { authorities: self.authorities.deserialize_over_with(de)? })
	}
}

// --- Inject
inventory::submit! {
	AuthInventory {
		get: |scheme, domain| {
			VFS.authorities.get(scheme, domain).map(|service| service.auth().clone())
		},
	}
}
