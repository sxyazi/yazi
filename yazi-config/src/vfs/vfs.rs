use std::io;

use anyhow::{Context, Result};
use mlua::{ExternalError, IntoLua, IntoLuaMulti, MetaMethod, UserData, UserDataMethods, Value};
use serde::{Deserialize, Deserializer, de::DeserializeSeed};
use yazi_codegen::DeserializeOver;
use yazi_fs::{Xdg, ok_or_not_found};
use yazi_shared::auth::{Auth, AuthInventory, Scheme};
use yazi_shim::toml::DeserializeOverWith;

use super::{Authorities, DomainSeed, Service, VfsMatcher};
use crate::VFS;

#[derive(Deserialize, DeserializeOver)]
pub struct Vfs {
	#[serde(flatten)]
	pub(super) authorities: Authorities,
}

impl Vfs {
	pub fn service<P>(auth: &Auth) -> io::Result<P>
	where
		P: TryFrom<Service, Error = &'static str>,
	{
		let Some(value) = VFS.authorities.service(&auth.scheme, &auth.domain) else {
			return Err(io::Error::other(format!("No such VFS service: {auth}")));
		};

		if value.kind() != auth.kind {
			return Err(io::Error::other(format!("VFS service `{auth}` kind changed")));
		}

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

impl UserData for &'static Vfs {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_meta_method(MetaMethod::Index, |lua, &me, scheme: Scheme| {
			match me.authorities.load().get(&scheme) {
				Some(domains) => domains.clone().into_lua(lua),
				None => Ok(Value::Nil),
			}
		});

		methods.add_meta_method(MetaMethod::NewIndex, |_, &me, (scheme, value): (Scheme, Value)| {
			match value {
				t @ Value::Table(_) => {
					let domains = DomainSeed(&scheme).deserialize(mlua::serde::Deserializer::new(t))?;
					me.authorities.insert(&scheme, &domains.into());
				}
				Value::Nil => me.authorities.remove(&scheme),
				_ => return Err("expected a table or nil".into_lua_err()),
			}
			Ok(())
		});

		methods.add_meta_method(MetaMethod::Pairs, |lua, &me, ()| {
			let mut matcher = VfsMatcher::from(&me.authorities);
			let iter = lua.create_function_mut(move |lua, ()| {
				if let Some((scheme, domains)) = matcher.next() {
					(scheme, domains).into_lua_multi(lua)
				} else {
					().into_lua_multi(lua)
				}
			})?;
			Ok(iter)
		});
	}
}

// --- Inject
inventory::submit! {
	AuthInventory {
		get: |scheme, domain| VFS.authorities.auth(scheme, domain),
	}
}
