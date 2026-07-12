use std::sync::Arc;

use serde::Deserialize;
use yazi_shared::auth::{Auth, AuthKind};

use super::{ServiceLua, ServiceSftp};

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum Service {
	Sftp(ServiceSftp),
	Mount(ServiceLua),
	Scope(ServiceLua),
}

impl TryFrom<&'static Service> for &'static ServiceSftp {
	type Error = &'static str;

	fn try_from(value: &'static Service) -> Result<Self, Self::Error> {
		match value {
			Service::Sftp(p) => Ok(p),
			Service::Mount(_) | Service::Scope(_) => {
				Err("expected an SFTP service, got a custom VFS service")
			}
		}
	}
}

impl TryFrom<&'static Service> for &'static ServiceLua {
	type Error = &'static str;

	fn try_from(value: &'static Service) -> Result<Self, Self::Error> {
		match value {
			Service::Sftp(_) => Err("expected a custom VFS service, got an SFTP service"),
			Service::Mount(lua) | Service::Scope(lua) => Ok(lua),
		}
	}
}

impl Service {
	pub fn kind(&self) -> AuthKind {
		match self {
			Self::Sftp(_) => AuthKind::Sftp,
			Self::Mount(_) => AuthKind::Mount,
			Self::Scope(_) => AuthKind::Scope,
		}
	}

	pub fn auth(&self) -> &Arc<Auth> {
		match self {
			Self::Sftp(sftp) => &sftp.auth,
			Self::Mount(lua) => &lua.auth,
			Self::Scope(lua) => &lua.auth,
		}
	}

	pub fn auth_mut(&mut self) -> &mut Arc<Auth> {
		match self {
			Self::Sftp(sftp) => &mut sftp.auth,
			Self::Mount(lua) => &mut lua.auth,
			Self::Scope(lua) => &mut lua.auth,
		}
	}
}
