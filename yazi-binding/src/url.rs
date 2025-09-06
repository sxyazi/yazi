use std::ops::Deref;

use mlua::{AnyUserData, ExternalError, FromLua, Lua, MetaMethod, UserData, UserDataFields, UserDataMethods, UserDataRef, Value};
use yazi_shared::{IntoOsStr, url::UrlCow};

use crate::{Urn, cached_field, deprecate};

pub type UrlRef = UserDataRef<Url>;

pub struct Url {
	inner: yazi_shared::url::UrlBuf,

	v_name:   Option<Value>,
	v_stem:   Option<Value>,
	v_ext:    Option<Value>,
	v_urn:    Option<Value>,
	v_base:   Option<Value>,
	v_parent: Option<Value>,
	v_domain: Option<Value>,
}

impl Deref for Url {
	type Target = yazi_shared::url::UrlBuf;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl AsRef<yazi_shared::url::UrlBuf> for Url {
	fn as_ref(&self) -> &yazi_shared::url::UrlBuf { &self.inner }
}

impl From<Url> for yazi_shared::url::UrlBuf {
	fn from(value: Url) -> Self { value.inner }
}

impl<'a> From<&'a Url> for yazi_shared::url::Url<'a> {
	fn from(value: &'a Url) -> Self { value.as_url() }
}

impl<'a> From<&'a Url> for UrlCow<'a> {
	fn from(value: &'a Url) -> Self {
		UrlCow::Borrowed { loc: value.loc.as_loc(), scheme: value.scheme.as_ref().into() }
	}
}

impl From<Url> for yazi_shared::url::UrlBufCov {
	fn from(value: Url) -> Self { Self(value.inner) }
}

impl TryFrom<&[u8]> for Url {
	type Error = mlua::Error;

	fn try_from(value: &[u8]) -> mlua::Result<Self> {
		Ok(Self::new(UrlCow::try_from(value)?.into_owned()))
	}
}

impl Url {
	pub fn new(url: impl Into<yazi_shared::url::UrlBuf>) -> Self {
		Self {
			inner: url.into(),

			v_name:   None,
			v_stem:   None,
			v_ext:    None,
			v_urn:    None,
			v_base:   None,
			v_parent: None,
			v_domain: None,
		}
	}

	pub fn install(lua: &Lua) -> mlua::Result<()> {
		lua.globals().raw_set(
			"Url",
			lua.create_function(|_, value: Value| {
				Ok(match value {
					Value::String(s) => Self::try_from(s.as_bytes().as_ref())?,
					Value::UserData(ud) => Self::new(ud.borrow::<Self>()?.inner.clone()),
					_ => Err("Expected a string or a Url".into_lua_err())?,
				})
			})?,
		)
	}
}

impl FromLua for Url {
	fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
		Ok(match value {
			Value::UserData(ud) => ud.take()?,
			_ => Err("Expected a Url".into_lua_err())?,
		})
	}
}

impl UserData for Url {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		cached_field!(fields, name, |lua, me| {
			me.name().map(|s| lua.create_string(s.as_encoded_bytes())).transpose()
		});
		cached_field!(fields, stem, |lua, me| {
			me.stem().map(|s| lua.create_string(s.as_encoded_bytes())).transpose()
		});
		cached_field!(fields, ext, |lua, me| {
			me.ext().map(|s| lua.create_string(s.as_encoded_bytes())).transpose()
		});
		cached_field!(fields, parent, |_, me| Ok(me.parent().map(Self::new)));
		cached_field!(fields, urn, |_, me| Ok(Urn::new(me.urn())));
		cached_field!(fields, base, |_, me| Ok(me.base().map(Self::new)));
		cached_field!(fields, domain, |lua, me| {
			me.scheme.domain().map(|s| lua.create_string(s)).transpose()
		});

		fields.add_field_method_get("frag", |lua, me| {
			deprecate!(lua, "`frag` property of Url is deprecated and renamed to `domain`, please use the new name instead, in your {}");
			me.scheme.domain().map(|s| lua.create_string(s)).transpose()
		});

		fields.add_field_method_get("is_regular", |_, me| Ok(me.is_regular()));
		fields.add_field_method_get("is_search", |_, me| Ok(me.is_search()));
		fields.add_field_method_get("is_archive", |_, me| Ok(me.is_archive()));
		fields.add_field_method_get("is_absolute", |_, me| Ok(me.is_absolute()));
		fields.add_field_method_get("has_root", |_, me| Ok(me.has_root()));
	}

	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_method("join", |_, me, other: Value| {
			Ok(Self::new(match other {
				Value::String(s) => me.join(s.as_bytes().into_os_str()?),
				Value::UserData(ud) => {
					let url = ud.borrow::<Self>()?;
					if !me.scheme.covariant(&url.scheme) {
						return Err("cannot join Urls with different schemes".into_lua_err());
					}
					me.join(&url.loc)
				}
				_ => Err("must be a string or Url".into_lua_err())?,
			}))
		});
		methods.add_method("starts_with", |_, me, base: Value| {
			Ok(match base {
				Value::String(s) => me.loc.starts_with(s.as_bytes().into_os_str()?),
				Value::UserData(ud) => me.starts_with(&*ud.borrow::<Self>()?),
				_ => Err("must be a string or Url".into_lua_err())?,
			})
		});
		methods.add_method("ends_with", |_, me, child: Value| {
			Ok(match child {
				Value::String(s) => me.loc.ends_with(s.as_bytes().into_os_str()?),
				Value::UserData(ud) => me.ends_with(&*ud.borrow::<Self>()?),
				_ => Err("must be a string or Url".into_lua_err())?,
			})
		});
		methods.add_method("strip_prefix", |_, me, base: Value| {
			let path = match base {
				Value::String(s) => me.loc.strip_prefix(s.as_bytes().into_os_str()?).ok(),
				Value::UserData(ud) => me.strip_prefix(&*ud.borrow::<Self>()?).map(AsRef::as_ref),
				_ => Err("must be a string or Url".into_lua_err())?,
			};
			Ok(path.map(Self::new)) // TODO: return `Path` instead of `Url`
		});

		methods.add_function_mut("into_search", |_, (ud, domain): (AnyUserData, mlua::String)| {
			Ok(Self::new(ud.take::<Self>()?.inner.into_search(domain.to_str()?)))
		});

		methods.add_meta_method(MetaMethod::Eq, |_, me, other: UrlRef| Ok(me.inner == other.inner));
		methods.add_meta_method(MetaMethod::ToString, |lua, me, ()| {
			lua.create_string(me.os_str().as_encoded_bytes())
		});
		methods.add_meta_method(MetaMethod::Concat, |lua, lhs, rhs: mlua::String| {
			lua.create_string([lhs.os_str().as_encoded_bytes(), &rhs.as_bytes()].concat())
		});
	}
}
