use mlua::{AnyUserData, Lua, UserDataFields, UserDataRef};

use super::{Cast, Cha};
use crate::url::Url;

pub type FileRef<'lua> = UserDataRef<'lua, yazi_shared::fs::File>;

pub struct File;

impl File {
	pub fn register(lua: &Lua) -> mlua::Result<()> {
		lua.register_userdata_type::<yazi_shared::fs::File>(|reg| {
			reg.add_field_method_get("url", |lua, me| Url::cast(lua, me.url.clone()));
			reg.add_field_method_get("cha", |lua, me| Cha::cast(lua, me.cha));
			reg.add_field_method_get("link_to", |lua, me| {
				me.link_to.as_ref().cloned().map(|u| Url::cast(lua, u)).transpose()
			});

			// Extension
			reg.add_field_method_get("name", |lua, me| {
				me.url.file_name().map(|n| lua.create_string(n.as_encoded_bytes())).transpose()
			});
		})
	}
}

impl<T: Into<yazi_shared::fs::File>> Cast<T> for File {
	fn cast(lua: &Lua, data: T) -> mlua::Result<AnyUserData> { lua.create_any_userdata(data.into()) }
}
