use std::io;

use anyhow::{Context, Result};
use serde::Deserialize;
use yazi_codegen::{DeserializeOver, DeserializeOver1};
use yazi_fs::{Xdg, ok_or_not_found};
use yazi_shared::auth::AuthInventory;

use super::{Service, Services};
use crate::VFS;

#[derive(Deserialize, DeserializeOver, DeserializeOver1)]
pub struct Vfs {
	pub services: Services,
}

impl Vfs {
	pub fn service<P>(domain: &str) -> io::Result<P>
	where
		P: TryFrom<&'static Service, Error = &'static str>,
	{
		let Some((key, value)) = VFS.services.get_key_value(domain) else {
			return Err(io::Error::other(format!("No such VFS service: {domain}")));
		};

		match value.try_into() {
			Ok(p) => Ok(p),
			Err(e) => Err(io::Error::other(format!("VFS service `{key}` has wrong kind: {e}"))),
		}
	}

	pub(crate) fn read() -> Result<String> {
		let p = Xdg::config_dir().join("vfs.toml");
		ok_or_not_found(std::fs::read_to_string(&p))
			.with_context(|| format!("Failed to read config {p:?}"))
	}
}

// --- Inject
inventory::submit! {
	AuthInventory {
		get: |scheme, domain| {
			VFS.services.get(domain).and_then(|service| {
				let auth = service.auth();
				if auth.scheme == scheme {
					Some(auth.clone())
				} else {
					None
				}
			})
		},
	}
}
