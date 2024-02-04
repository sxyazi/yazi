use mlua::{IntoLuaMulti, Lua, Table, Value};
use yazi_adaptor::{Image, ADAPTOR};

use super::Utils;
use crate::{elements::RectRef, url::UrlRef};

impl Utils {
	pub(super) fn image(lua: &Lua, ya: &Table) -> mlua::Result<()> {
		ya.set(
			"image_show",
			lua.create_async_function(|lua, (url, rect): (UrlRef, RectRef)| async move {
				if let Ok(size) = ADAPTOR.image_show(&url, *rect).await {
					size.into_lua_multi(lua)
				} else {
					Value::Nil.into_lua_multi(lua)
				}
			})?,
		)?;

		ya.set(
			"image_precache",
			lua.create_async_function(|_, (src, dist): (UrlRef, UrlRef)| async move {
				Ok(Image::precache(&src, dist.to_path_buf()).await.is_ok())
			})?,
		)?;

		Ok(())
	}
}
