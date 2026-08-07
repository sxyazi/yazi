use mlua::{Function, IntoLuaMulti, Lua, Value};
use yazi_adapter::{ADAPTOR, Image};
use yazi_binding::{ImageInfo, elements::Rect};
use yazi_shared::url::{UrlLike, UrlRef};
use yazi_shim::{OptionExt, fs::Error};

use super::Utils;

impl Utils {
	pub(super) fn image_info(lua: &Lua) -> mlua::Result<Function> {
		lua.create_async_function(|lua, url: UrlRef| async move {
			let Some(path) = url.as_local().owned() else {
				return (Value::Nil, Error::other("Source must be a local path")).into_lua_multi(&lua);
			};

			match ImageInfo::new(path).await {
				Ok(info) => info.into_lua_multi(&lua),
				Err(e) => (Value::Nil, Error::other(e.to_string())).into_lua_multi(&lua),
			}
		})
	}

	pub(super) fn image_show(lua: &Lua) -> mlua::Result<Function> {
		lua.create_async_function(|lua, (url, rect): (UrlRef, Rect)| async move {
			let Some(path) = url.as_local() else {
				return (Value::Nil, Error::other("Source must be a local path")).into_lua_multi(&lua);
			};

			match ADAPTOR.image_show(path, *rect).await {
				Ok(area) => Rect::from(area).into_lua_multi(&lua),
				Err(e) => (Value::Nil, Error::other(e.to_string())).into_lua_multi(&lua),
			}
		})
	}

	pub(super) fn image_precache(lua: &Lua) -> mlua::Result<Function> {
		lua.create_async_function(|lua, (src, dist): (UrlRef, UrlRef)| async move {
			let Some(src) = src.as_local().owned() else {
				return (false, Error::other("Source must be a local path")).into_lua_multi(&lua);
			};
			let Some(dist) = dist.as_local() else {
				return (false, Error::other("Destination must be a local path")).into_lua_multi(&lua);
			};

			match Image::precache(src, dist).await {
				Ok(()) => true.into_lua_multi(&lua),
				Err(e) => (false, Error::other(e.to_string())).into_lua_multi(&lua),
			}
		})
	}
}
