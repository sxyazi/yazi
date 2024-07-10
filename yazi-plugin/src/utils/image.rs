use mlua::{IntoLuaMulti, Lua, Table, Value};
use yazi_adapter::{Image, ADAPTOR};

use super::Utils;
use crate::{bindings::Cast, elements::{Rect, RectRef}, url::UrlRef};

impl Utils {
	pub(super) fn image(lua: &Lua, ya: &Table) -> mlua::Result<()> {
		ya.raw_set(
			"image_show",
			lua.create_async_function(|lua, (url, rect): (UrlRef, RectRef)| async move {
				if let Ok(area) = ADAPTOR.image_show(&url, *rect).await {
					Rect::cast(lua, area)?.into_lua_multi(lua)
				} else {
					Value::Nil.into_lua_multi(lua)
				}
			})?,
		)?;

		ya.raw_set(
			"image_precache",
			lua.create_async_function(|_, (src, dist): (UrlRef, UrlRef)| async move {
				Ok(Image::precache(&src, dist.to_path_buf()).await.is_ok())
			})?,
		)?;

		Ok(())
	}
}
