use mlua::{AnyUserData, Lua, MetaMethod, UserDataFields, UserDataMethods, UserDataRef};

use crate::bindings::Cast;

pub type UrlRef<'lua> = UserDataRef<'lua, yazi_shared::fs::Url>;

pub struct Url;

impl Url {
	pub fn register(lua: &Lua) -> mlua::Result<()> {
		lua.register_userdata_type::<yazi_shared::fs::Url>(|reg| {
			reg.add_field_method_get("frag", |_, me| Ok(me.frag().map(ToOwned::to_owned)));
			reg.add_field_method_get("is_regular", |_, me| Ok(me.is_regular()));
			reg.add_field_method_get("is_search", |_, me| Ok(me.is_search()));
			reg.add_field_method_get("is_archive", |_, me| Ok(me.is_archive()));

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
		lua.globals().set(
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
