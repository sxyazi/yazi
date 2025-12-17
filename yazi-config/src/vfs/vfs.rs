use std::io;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tokio::sync::OnceCell;
use yazi_codegen::DeserializeOver1;
use yazi_fs::{Xdg, ok_or_not_found};

use super::Service;
use crate::{Preset, vfs::Services};

#[derive(Deserialize, Serialize, DeserializeOver1)]
pub struct Vfs {
	pub services: Services,
}

impl Vfs {
	pub async fn load() -> io::Result<&'static Self> {
		pub static LOADED: OnceCell<Vfs> = OnceCell::const_new();

		async fn init() -> io::Result<Vfs> {
			tokio::task::spawn_blocking(|| {
				Preset::vfs()?.deserialize_over(toml::Deserializer::parse(&Vfs::read()?)?)?.reshape()
			})
			.await?
			.map_err(io::Error::other)
		}

		LOADED.get_or_try_init(init).await
	}

	pub async fn service<'a, P>(name: &str) -> io::Result<(&'a str, P)>
	where
		P: TryFrom<&'a Service, Error = &'static str>,
	{
		let Some((key, value)) = Self::load().await?.services.get_key_value(name) else {
			return Err(io::Error::other(format!("No such VFS service: {name}")));
		};
		match value.try_into() {
			Ok(p) => Ok((key.as_str(), p)),
			Err(e) => Err(io::Error::other(format!("VFS service `{key}` has wrong type: {e}"))),
		}
	}

	fn read() -> Result<String> {
		let p = Xdg::config_dir().join("vfs.toml");
		ok_or_not_found(std::fs::read_to_string(&p))
			.with_context(|| format!("Failed to read config {p:?}"))
	}

	fn reshape(self) -> Result<Self> { Ok(Self { services: self.services.reshape()? }) }
}
