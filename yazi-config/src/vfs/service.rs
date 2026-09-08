use std::sync::Arc;

use mlua::{FromLua, IntoLua, Lua, LuaSerdeExt, LuaString, MetaMethod, Table, UserData, UserDataFields, UserDataMethods, Value};
use serde::Deserialize;
use yazi_shared::{auth::{AuthArc, AuthKind, Scheme}, path::DynPath, sendable::Sendable};
use yazi_shim::{mlua::UserDataFieldsExt, strum::IntoStr};

use super::{ServiceLua, ServiceSftp};

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum Service {
	Sftp(Arc<ServiceSftp>),
	Mount(Arc<ServiceLua>),
	Hub(Arc<ServiceLua>),
	Scope(Arc<ServiceLua>),
	View(Arc<ServiceLua>),
}

impl TryFrom<Service> for Arc<ServiceSftp> {
	type Error = &'static str;

	fn try_from(value: Service) -> Result<Self, Self::Error> {
		match value {
			Service::Sftp(p) => Ok(p),
			Service::Mount(_) | Service::Hub(_) | Service::Scope(_) | Service::View(_) => {
				Err("expected an SFTP service, got a custom VFS service")
			}
		}
	}
}

impl TryFrom<Service> for Arc<ServiceLua> {
	type Error = &'static str;

	fn try_from(value: Service) -> Result<Self, Self::Error> {
		match value {
			Service::Sftp(_) => Err("expected a custom VFS service, got an SFTP service"),
			Service::Mount(lua) | Service::Hub(lua) | Service::Scope(lua) | Service::View(lua) => Ok(lua),
		}
	}
}

impl Service {
	pub(crate) fn configure<D>(&mut self, scheme: &Scheme, domain: &D) -> Result<(), &'static str>
	where
		D: AsRef<[u8]> + ?Sized,
	{
		let kind = self.kind();
		if scheme.is_regular() {
			return Err("scheme cannot be configured");
		} else if kind.is_sftp() != scheme.is_sftp() {
			return Err("service kind does not match scheme");
		} else if kind.is_hub() && domain.as_ref() != b"*" {
			return Err("hub services require a `*` catch-all domain");
		}

		*self.auth_make_mut() = AuthArc::new(kind, scheme.clone(), domain.as_ref());
		Ok(())
	}

	pub(crate) fn kind(&self) -> AuthKind {
		match self {
			Self::Sftp(_) => AuthKind::Sftp,
			Self::Mount(_) => AuthKind::Mount,
			Self::Hub(_) => AuthKind::Hub,
			Self::Scope(_) => AuthKind::Scope,
			Self::View(_) => AuthKind::View,
		}
	}

	pub(crate) fn auth(&self) -> &AuthArc {
		match self {
			Self::Sftp(sftp) => &sftp.auth,
			Self::Mount(lua) | Self::Hub(lua) | Self::Scope(lua) | Self::View(lua) => &lua.auth,
		}
	}

	fn auth_make_mut(&mut self) -> &mut AuthArc {
		match self {
			Self::Sftp(sftp) => &mut Arc::make_mut(sftp).auth,
			Self::Mount(lua) | Self::Hub(lua) | Self::Scope(lua) | Self::View(lua) => {
				&mut Arc::make_mut(lua).auth
			}
		}
	}
}

impl FromLua for Service {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		let t = Table::from_lua(value, lua)?;
		let kind: Option<LuaString> = t.raw_get("kind")?;

		if kind.is_some() {
			lua.from_value(Value::Table(t))
		} else {
			let sftp: ServiceSftp = lua.from_value(Value::Table(t))?;
			Ok(Self::Sftp(sftp.into()))
		}
	}
}

impl UserData for Service {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_cached_field("kind", |_, me| Ok(me.kind().into_str()));
	}

	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_meta_method(MetaMethod::Index, |lua, me, key: LuaString| {
			let key = key.as_bytes();
			match me {
				Self::Sftp(s) => match &*key {
					b"host" => lua.create_string(&s.host)?.into_lua(lua),
					b"user" => lua.create_string(&s.user)?.into_lua(lua),
					b"port" => s.port.into_lua(lua),
					b"password" => s.password.as_deref().into_lua(lua),
					b"key_file" => s.key_file.dyn_path().into_lua(lua),
					b"key_passphrase" => s.key_passphrase.as_deref().into_lua(lua),
					b"cert_file" => s.cert_file.dyn_path().into_lua(lua),
					b"no_cert_verify" => s.no_cert_verify.into_lua(lua),
					b"identity_agent" => s.identity_agent.dyn_path().into_lua(lua),
					_ => Ok(Value::Nil),
				},
				Self::Mount(s) | Self::Hub(s) | Self::Scope(s) | Self::View(s) => match &*key {
					b"name" => lua.create_string(&*s.name)?.into_lua(lua),
					b"args" => Sendable::args_to_table_ref(lua, &s.args)?.into_lua(lua),
					b"opts" => Sendable::args_to_table_ref(lua, &s.opts)?.into_lua(lua),
					_ => Ok(Value::Nil),
				},
			}
		});
	}
}
