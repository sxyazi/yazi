use mlua::{AnyUserData, ExternalError, Lua, MetaMethod, UserDataFields, UserDataMethods, UserDataRef, Value};

use crate::bindings::Cast;

pub type UrlRef = UserDataRef<yazi_shared::fs::Url>;

pub struct Url;

impl Url {
	pub fn register(lua: &Lua) -> mlua::Result<()> {
		lua.register_userdata_type::<yazi_shared::fs::Url>(|reg| {
			reg.add_method("frag", |lua, me, ()| lua.create_string(me.frag()));
			reg.add_field_method_get("is_regular", |_, me| Ok(me.is_regular()));
			reg.add_field_method_get("is_search", |_, me| Ok(me.is_search()));
			reg.add_field_method_get("is_archive", |_, me| Ok(me.is_archive()));
			reg.add_field_method_get("is_absolute", |_, me| Ok(me.is_absolute()));
			reg.add_field_method_get("has_root", |_, me| Ok(me.has_root()));

			reg.add_method("name", |lua, me, ()| {
				me.file_name().map(|s| lua.create_string(s.as_encoded_bytes())).transpose()
			});
			reg.add_method("stem", |lua, me, ()| {
				me.file_stem().map(|s| lua.create_string(s.as_encoded_bytes())).transpose()
			});
			reg.add_method("ext", |lua, me, ()| {
				me.extension().map(|s| lua.create_string(s.as_encoded_bytes())).transpose()
			});
			reg.add_method("join", |lua, me, other: Value| {
				Self::cast(lua, match other {
					Value::String(s) => me.join(s.to_str()?.as_ref()),
					Value::UserData(ud) => me.join(&*ud.borrow::<yazi_shared::fs::Url>()?),
					_ => Err("must be a string or a Url".into_lua_err())?,
				})
			});
			reg.add_method("parent", |lua, me, ()| {
				me.parent_url().map(|u| Self::cast(lua, u)).transpose()
			});
			reg.add_method("starts_with", |_, me, base: Value| {
				Ok(match base {
					Value::String(s) => me.starts_with(s.to_str()?.as_ref()),
					Value::UserData(ud) => me.starts_with(&*ud.borrow::<yazi_shared::fs::Url>()?),
					_ => Err("must be a string or a Url".into_lua_err())?,
				})
			});
			reg.add_method("ends_with", |_, me, child: Value| {
				Ok(match child {
					Value::String(s) => me.ends_with(s.to_str()?.as_ref()),
					Value::UserData(ud) => me.ends_with(&*ud.borrow::<yazi_shared::fs::Url>()?),
					_ => Err("must be a string or a Url".into_lua_err())?,
				})
			});
			reg.add_method("strip_prefix", |lua, me, base: Value| {
				let path = match base {
					Value::String(s) => me.strip_prefix(s.to_str()?.as_ref()),
					Value::UserData(ud) => me.strip_prefix(&*ud.borrow::<yazi_shared::fs::Url>()?),
					_ => Err("must be a string or a Url".into_lua_err())?,
				};
				path.ok().map(|p| Self::cast(lua, yazi_shared::fs::Url::from(p))).transpose()
			});

			reg.add_meta_method(MetaMethod::Eq, |_, me, other: UrlRef| Ok(me == &*other));
			reg.add_meta_method(MetaMethod::ToString, |lua, me, ()| {
				lua.create_string(me.as_os_str().as_encoded_bytes())
			});
			reg.add_meta_method(MetaMethod::Concat, |lua, me, other: mlua::String| {
				let me = me.as_os_str().as_encoded_bytes();
				let other = other.as_bytes();

				let mut out = Vec::with_capacity(me.len() + other.len());
				out.extend_from_slice(me);
				out.extend_from_slice(&other);
				lua.create_string(out)
			});
		})
	}

	pub fn install(lua: &Lua) -> mlua::Result<()> {
		lua.globals().raw_set(
			"Url",
			lua.create_function(|lua, url: mlua::String| {
				Self::cast(lua, yazi_shared::fs::Url::from(url.to_str()?.as_ref()))
			})?,
		)
	}
}

impl<T: Into<yazi_shared::fs::Url>> Cast<T> for Url {
	fn cast(lua: &Lua, data: T) -> mlua::Result<AnyUserData> { lua.create_any_userdata(data.into()) }
}
