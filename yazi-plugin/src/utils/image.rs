use mlua::{IntoLua, Lua, Table, Value};
use yazi_adapter::{ADAPTOR, Image};

use super::Utils;
use crate::{elements::Rect, url::UrlRef};

impl Utils {
	pub(super) fn image(lua: &Lua, ya: &Table) -> mlua::Result<()> {
		ya.raw_set(
			"image_show",
			lua.create_async_function(|lua, (url, rect): (UrlRef, Rect)| async move {
				if let Ok(area) = ADAPTOR.image_show(&url, *rect).await {
					Rect::from(area).into_lua(&lua)
				} else {
					Value::Nil.into_lua(&lua)
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
