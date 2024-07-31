use mlua::{AnyUserData, ExternalError, Lua, MetaMethod, UserDataFields, UserDataMethods, UserDataRef, Value};

use crate::bindings::Cast;

pub type UrlRef<'lua> = UserDataRef<'lua, yazi_shared::fs::Url>;

pub struct Url;

impl Url {
	pub fn register(lua: &Lua) -> mlua::Result<()> {
		lua.register_userdata_type::<yazi_shared::fs::Url>(|reg| {
			reg.add_method("frag", |lua, me, ()| lua.create_string(me.frag()));
			reg.add_field_method_get("is_regular", |_, me| Ok(me.is_regular()));
			reg.add_field_method_get("is_search", |_, me| Ok(me.is_search()));
			reg.add_field_method_get("is_archive", |_, me| Ok(me.is_archive()));

			reg.add_method("name", |lua, me, ()| {
				me.file_name().map(|s| lua.create_string(s.as_encoded_bytes())).transpose()
			});
			reg.add_method("stem", |lua, me, ()| {
				me.file_stem().map(|s| lua.create_string(s.as_encoded_bytes())).transpose()
			});
			reg.add_method("join", |lua, me, other: Value| {
				Ok(match other {
					Value::String(s) => Self::cast(lua, me.join(s.to_str()?)),
					Value::UserData(ud) => {
						let url = ud.borrow::<yazi_shared::fs::Url>()?;
						Self::cast(lua, me.join(&*url))
					}
					_ => Err("must be a string or a Url".into_lua_err())?,
				})
			});
			reg.add_method("parent", |lua, me, ()| {
				me.parent_url().map(|u| Self::cast(lua, u)).transpose()
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
				out.extend_from_slice(other);
				lua.create_string(out)
			});
		})
	}

	pub fn install(lua: &Lua) -> mlua::Result<()> {
		lua.globals().raw_set(
			"Url",
			lua.create_function(|lua, url: mlua::String| {
				Self::cast(lua, yazi_shared::fs::Url::from(url.to_str()?))
			})?,
		)
	}
}

impl<T: Into<yazi_shared::fs::Url>> Cast<T> for Url {
	fn cast(lua: &Lua, data: T) -> mlua::Result<AnyUserData> { lua.create_any_userdata(data.into()) }
}
