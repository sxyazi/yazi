use mlua::{Function, IntoLua, Lua, Value};
use yazi_adapter::{ADAPTOR, Image};
use yazi_binding::UrlRef;

use super::Utils;
use crate::{bindings::ImageInfo, elements::Rect};

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
		lua.create_async_function(|lua, (url, rect): (UrlRef, Rect)| async move {
			if let Ok(area) = ADAPTOR.get().image_show(&url, *rect).await {
				Rect::from(area).into_lua(&lua)
			} else {
				Value::Nil.into_lua(&lua)
			}
		})
	}

	pub(super) fn image_precache(lua: &Lua) -> mlua::Result<Function> {
		lua.create_async_function(|_, (src, dist): (UrlRef, UrlRef)| async move {
			Ok(Image::precache(&src, dist.to_path_buf()).await.is_ok())
		})
	}
}
