use mlua::{AnyUserData, ExternalError, FromLua, IntoLua, Lua, MetaMethod, UserDataFields, UserDataMethods, UserDataRef, Value};

pub type UrlRef = UserDataRef<yazi_shared::url::Url>;

pub struct Url(pub yazi_shared::url::Url);

impl Url {
	pub fn register(lua: &Lua) -> mlua::Result<()> {
		lua.register_userdata_type::<yazi_shared::url::Url>(|reg| {
			reg.add_method("frag", |lua, me, ()| lua.create_string(me.frag()));
			reg.add_field_method_get("base", |_, me| {
				Ok(if me.base().as_os_str().is_empty() { None } else { Some(Self::from(me.base())) })
			});
			reg.add_field_method_get("is_regular", |_, me| Ok(me.is_regular()));
			reg.add_field_method_get("is_search", |_, me| Ok(me.is_search()));
			reg.add_field_method_get("is_archive", |_, me| Ok(me.is_archive()));
			reg.add_field_method_get("is_absolute", |_, me| Ok(me.is_absolute()));
			reg.add_field_method_get("has_root", |_, me| Ok(me.has_root()));

			reg.add_method("name", |lua, me, ()| {
				Some(me.name())
					.filter(|&s| !s.is_empty())
					.map(|s| lua.create_string(s.as_encoded_bytes()))
					.transpose()
			});
			reg.add_method("stem", |lua, me, ()| {
				me.file_stem().map(|s| lua.create_string(s.as_encoded_bytes())).transpose()
			});
			reg.add_method("ext", |lua, me, ()| {
				me.extension().map(|s| lua.create_string(s.as_encoded_bytes())).transpose()
			});
			reg.add_method("join", |_, me, other: Value| {
				Ok(Self(match other {
					Value::String(s) => me.join(s.to_str()?.as_ref()),
					Value::UserData(ud) => me.join(&*ud.borrow::<yazi_shared::url::Url>()?),
					_ => Err("must be a string or a Url".into_lua_err())?,
				}))
			});
			reg.add_method("parent", |_, me, ()| Ok(me.parent_url().map(Self)));
			reg.add_method("starts_with", |_, me, base: Value| {
				Ok(match base {
					Value::String(s) => me.starts_with(s.to_str()?.as_ref()),
					Value::UserData(ud) => me.starts_with(&*ud.borrow::<yazi_shared::url::Url>()?),
					_ => Err("must be a string or a Url".into_lua_err())?,
				})
			});
			reg.add_method("ends_with", |_, me, child: Value| {
				Ok(match child {
					Value::String(s) => me.ends_with(s.to_str()?.as_ref()),
					Value::UserData(ud) => me.ends_with(&*ud.borrow::<yazi_shared::url::Url>()?),
					_ => Err("must be a string or a Url".into_lua_err())?,
				})
			});
			reg.add_method("strip_prefix", |_, me, base: Value| {
				let path = match base {
					Value::String(s) => me.strip_prefix(s.to_str()?.as_ref()),
					Value::UserData(ud) => me.strip_prefix(&*ud.borrow::<yazi_shared::url::Url>()?),
					_ => Err("must be a string or a Url".into_lua_err())?,
				};
				Ok(path.ok().map(Self::from))
			});

			reg.add_function_mut("into_search", |_, (ud, frag): (AnyUserData, mlua::String)| {
				Ok(Self(ud.take::<yazi_shared::url::Url>()?.into_search(&frag.to_str()?)))
			});

			reg.add_meta_method(MetaMethod::Eq, |_, me, other: UrlRef| Ok(me == &*other));
			reg.add_meta_method(MetaMethod::ToString, |lua, me, ()| {
				lua.create_string(me.as_os_str().as_encoded_bytes())
			});
			reg.add_meta_method(MetaMethod::Concat, |lua, lhs, rhs: mlua::String| {
				lua.create_string([lhs.as_os_str().as_encoded_bytes(), rhs.as_bytes().as_ref()].concat())
			});
		})
	}

	pub fn install(lua: &Lua) -> mlua::Result<()> {
		lua.globals().raw_set(
			"Url",
			lua.create_function(|_, value: Value| {
				Ok(match value {
					Value::String(s) => Self::from(s.to_str()?.as_ref()),
					Value::UserData(ud) => Self(ud.borrow::<yazi_shared::url::Url>()?.clone()),
					_ => Err("Expected a string or a Url".into_lua_err())?,
				})
			})?,
		)
	}
}

impl<T: Into<yazi_shared::url::Url>> From<T> for Url {
	fn from(value: T) -> Self { Self(value.into()) }
}

impl FromLua for Url {
	fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
		match value {
			Value::UserData(ud) => ud.take().map(Self),
			_ => Err("Expected a Url".into_lua_err()),
		}
	}
}

impl IntoLua for Url {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		lua.create_any_userdata(self.0)?.into_lua(lua)
	}
}
