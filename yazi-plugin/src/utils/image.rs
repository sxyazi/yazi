use mlua::{Function, IntoLuaMulti, Lua, Value};
use yazi_adapter::{ADAPTOR, Image};
use yazi_binding::{Error, UrlRef, elements::Rect};

use super::Utils;
use crate::bindings::ImageInfo;

impl Utils {
	pub(super) fn image_info(lua: &Lua) -> mlua::Result<Function> {
		lua.create_async_function(|lua, url: UrlRef| async move {
			let Some(path) = url.as_path() else {
				return (Value::Nil, Error::custom("Only local files are supported")).into_lua_multi(&lua);
			};
			match yazi_adapter::ImageInfo::new(path).await {
				Ok(info) => ImageInfo::from(info).into_lua_multi(&lua),
				Err(e) => (Value::Nil, Error::custom(e.to_string())).into_lua_multi(&lua),
			}
		})
	}

	pub(super) fn image_show(lua: &Lua) -> mlua::Result<Function> {
		lua.create_async_function(|lua, (url, rect): (UrlRef, Rect)| async move {
			let Some(path) = url.as_path() else {
				return (Value::Nil, Error::custom("Only local files are supported")).into_lua_multi(&lua);
			};
			match ADAPTOR.get().image_show(path, *rect).await {
				Ok(area) => Rect::from(area).into_lua_multi(&lua),
				Err(e) => (Value::Nil, Error::custom(e.to_string())).into_lua_multi(&lua),
			}
		})
	}

	pub(super) fn image_precache(lua: &Lua) -> mlua::Result<Function> {
		lua.create_async_function(|lua, (src, dist): (UrlRef, UrlRef)| async move {
			let Some((src, dist)) = src.as_path().zip(dist.as_path()) else {
				return (Value::Nil, Error::custom("Only local files are supported")).into_lua_multi(&lua);
			};
			match Image::precache(src, dist).await {
				Ok(()) => true.into_lua_multi(&lua),
				Err(e) => (false, Error::custom(e.to_string())).into_lua_multi(&lua),
			}
		})
	}
}
