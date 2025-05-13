use std::{ops::Deref, path::Path};

use mlua::{AnyUserData, ExternalError, FromLua, Lua, MetaMethod, UserData, UserDataFields, UserDataMethods, UserDataRef, Value};

use crate::{Urn, cached_field};

pub type UrlRef = UserDataRef<Url>;

pub struct Url {
	inner: yazi_shared::url::Url,

	v_name:   Option<Value>,
	v_stem:   Option<Value>,
	v_ext:    Option<Value>,
	v_urn:    Option<Value>,
	v_base:   Option<Value>,
	v_parent: Option<Value>,
	v_frag:   Option<Value>,
}

impl Deref for Url {
	type Target = yazi_shared::url::Url;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl AsRef<Path> for Url {
	fn as_ref(&self) -> &Path { self.inner.as_path() }
}

impl From<Url> for yazi_shared::url::Url {
	fn from(value: Url) -> Self { value.inner }
}

impl Url {
	pub fn new(url: impl Into<yazi_shared::url::Url>) -> Self {
		Self {
			inner: url.into(),

			v_name:   None,
			v_stem:   None,
			v_ext:    None,
			v_urn:    None,
			v_base:   None,
			v_parent: None,
			v_frag:   None,
		}
	}

	pub fn install(lua: &Lua) -> mlua::Result<()> {
		lua.globals().raw_set(
			"Url",
			lua.create_function(|_, value: Value| {
				Ok(match value {
					Value::String(s) => Self::new(s.to_str()?.as_ref()),
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
			Some(me.name())
				.filter(|&s| !s.is_empty())
				.map(|s| lua.create_string(s.as_encoded_bytes()))
				.transpose()
		});
		cached_field!(fields, stem, |lua, me| {
			me.file_stem().map(|s| lua.create_string(s.as_encoded_bytes())).transpose()
		});
		cached_field!(fields, ext, |lua, me| {
			me.extension().map(|s| lua.create_string(s.as_encoded_bytes())).transpose()
		});
		cached_field!(fields, parent, |_, me| Ok(me.parent_url().map(Self::new)));
		cached_field!(fields, urn, |_, me| Ok(Urn::new(me.urn_owned())));
		cached_field!(fields, base, |_, me| {
			Ok(if me.base().as_os_str().is_empty() { None } else { Some(Self::new(me.base())) })
		});
		cached_field!(fields, frag, |lua, me| lua.create_string(me.frag()));

		fields.add_field_method_get("is_regular", |_, me| Ok(me.is_regular()));
		fields.add_field_method_get("is_search", |_, me| Ok(me.is_search()));
		fields.add_field_method_get("is_archive", |_, me| Ok(me.is_archive()));
		fields.add_field_method_get("is_absolute", |_, me| Ok(me.is_absolute()));
		fields.add_field_method_get("has_root", |_, me| Ok(me.has_root()));
	}

	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_method("join", |_, me, other: Value| {
			Ok(Self::new(match other {
				Value::String(s) => me.join(s.to_str()?.as_ref()),
				Value::UserData(ud) => me.join(&ud.borrow::<Self>()?.inner),
				_ => Err("must be a string or a Url".into_lua_err())?,
			}))
		});
		methods.add_method("starts_with", |_, me, base: Value| {
			Ok(match base {
				Value::String(s) => me.starts_with(s.to_str()?.as_ref()),
				Value::UserData(ud) => me.starts_with(&ud.borrow::<Self>()?.inner),
				_ => Err("must be a string or a Url".into_lua_err())?,
			})
		});
		methods.add_method("ends_with", |_, me, child: Value| {
			Ok(match child {
				Value::String(s) => me.ends_with(s.to_str()?.as_ref()),
				Value::UserData(ud) => me.ends_with(&ud.borrow::<Self>()?.inner),
				_ => Err("must be a string or a Url".into_lua_err())?,
			})
		});
		methods.add_method("strip_prefix", |_, me, base: Value| {
			let path = match base {
				Value::String(s) => me.strip_prefix(s.to_str()?.as_ref()),
				Value::UserData(ud) => me.strip_prefix(&ud.borrow::<Self>()?.inner),
				_ => Err("must be a string or a Url".into_lua_err())?,
			};
			Ok(path.ok().map(Self::new))
		});

		methods.add_function_mut("into_search", |_, (ud, frag): (AnyUserData, mlua::String)| {
			Ok(Self::new(ud.take::<Self>()?.inner.into_search(&frag.to_str()?)))
		});

		methods.add_meta_method(MetaMethod::Eq, |_, me, other: UrlRef| Ok(me.inner == other.inner));
		methods.add_meta_method(MetaMethod::ToString, |lua, me, ()| {
			lua.create_string(me.as_os_str().as_encoded_bytes())
		});
		methods.add_meta_method(MetaMethod::Concat, |lua, lhs, rhs: mlua::String| {
			lua.create_string([lhs.as_os_str().as_encoded_bytes(), &rhs.as_bytes()].concat())
		});
	}
}
