use mlua::{Function, IntoLuaMulti, Lua, Value};
use yazi_adapter::{ADAPTOR, Image};
use yazi_binding::{Error, UrlRef};

use super::Utils;
use crate::{bindings::ImageInfo, elements::Rect};

impl Utils {
	pub(super) fn image_info(lua: &Lua) -> mlua::Result<Function> {
		lua.create_async_function(|lua, url: UrlRef| async move {
			match yazi_adapter::ImageInfo::new(&url).await {
				Ok(info) => ImageInfo::from(info).into_lua_multi(&lua),
				Err(e) => (Value::Nil, Error::Custom(e.to_string().into())).into_lua_multi(&lua),
			}
		})
	}

	pub(super) fn image_show(lua: &Lua) -> mlua::Result<Function> {
		lua.create_async_function(|lua, (url, rect): (UrlRef, Rect)| async move {
			match ADAPTOR.get().image_show(&url, *rect).await {
				Ok(area) => Rect::from(area).into_lua_multi(&lua),
				Err(e) => (Value::Nil, Error::Custom(e.to_string().into())).into_lua_multi(&lua),
			}
		})
	}

	pub(super) fn image_precache(lua: &Lua) -> mlua::Result<Function> {
		lua.create_async_function(|lua, (src, dist): (UrlRef, UrlRef)| async move {
			match Image::precache(&src, &dist).await {
				Ok(()) => true.into_lua_multi(&lua),
				Err(e) => (false, Error::Custom(e.to_string().into())).into_lua_multi(&lua),
			}
		})
	}
}
