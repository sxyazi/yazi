use mlua::{Function, IntoLua, Lua, Table, Value};
use yazi_adapter::{ADAPTOR, Image, Offset};

use super::Utils;
use crate::{bindings::ImageInfo, elements::Rect, url::UrlRef};

impl Utils {
	pub(super) fn image_info(lua: &Lua) -> mlua::Result<Function> {
		lua.create_async_function(|lua, url: UrlRef| async move {
			if let Ok(info) = yazi_adapter::ImageInfo::new(&url).await {
				ImageInfo::from(info).into_lua(&lua)
			} else {
				Value::Nil.into_lua(&lua)
			}
		})
	}

	pub(super) fn image_show(lua: &Lua) -> mlua::Result<Function> {
		lua.create_async_function(
			|lua, (url, rect, offset_table): (UrlRef, Rect, Option<Table>)| async move {
				let offset = offset_table.map(|lua_offset| Offset {
					x: lua_offset.get("x").unwrap_or(0),
					y: lua_offset.get("y").unwrap_or(0),
				});
				if let Ok(area) = ADAPTOR.image_show(&url, *rect, offset).await {
					Rect::from(area).into_lua(&lua)
				} else {
					Value::Nil.into_lua(&lua)
				}
			},
		)
	}

	pub(super) fn image_precache(lua: &Lua) -> mlua::Result<Function> {
		lua.create_async_function(|_, (src, dist): (UrlRef, UrlRef)| async move {
			Ok(Image::precache(&src, dist.to_path_buf()).await.is_ok())
		})
	}
}
