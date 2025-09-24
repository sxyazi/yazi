use std::io;

use hashbrown::HashMap;
use serde::{Deserialize, Serialize};
use tokio::sync::OnceCell;
use yazi_macro::ok_or_not_found;

use super::Provider;
use crate::local::Xdg;

#[derive(Deserialize, Serialize)]
pub struct Vfs {
	pub providers: HashMap<String, Provider>,
}

impl Vfs {
	pub async fn load() -> io::Result<&'static Self> {
		pub static LOADED: OnceCell<Vfs> = OnceCell::const_new();

		async fn init() -> io::Result<Vfs> {
			tokio::task::spawn_blocking(|| {
				toml::from_str::<Vfs>(&Vfs::read()?).map_err(io::Error::other)?.reshape()
			})
			.await?
		}

		LOADED.get_or_try_init(init).await
	}

	pub async fn provider<'a, P>(name: &str) -> io::Result<(&'a str, P)>
	where
		P: TryFrom<&'a Provider, Error = &'static str>,
	{
		let Some((key, value)) = Self::load().await?.providers.get_key_value(name) else {
			return Err(io::Error::other(format!("No such VFS provider: {name}")));
		};
		match value.try_into() {
			Ok(p) => Ok((key.as_str(), p)),
			Err(e) => Err(io::Error::other(format!("VFS provider `{key}` has wrong type: {e}"))),
		}
	}

	fn read() -> io::Result<String> {
		let p = Xdg::config_dir().join("vfs.toml");
		Ok(ok_or_not_found!(std::fs::read_to_string(&p).map_err(|e| {
			std::io::Error::new(e.kind(), format!("Failed to read VFS config {p:?}: {e}"))
		})))
	}

	fn reshape(self) -> io::Result<Self> {
		for name in self.providers.keys() {
			if name.is_empty() || name.len() > 20 {
				Err(io::Error::other(format!("VFS name `{name}` must be between 1 and 20 characters")))?;
			} else if !name.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'z' | b'-')) {
				Err(io::Error::other(format!("VFS name `{name}` must be in kebab-case")))?;
			}
		}

		Ok(self)
	}
}
