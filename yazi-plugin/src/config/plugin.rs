use mlua::{Function, IntoLua, Lua, UserData, Value};
use yazi_binding::{UrlRef, cached_field};
use yazi_config::YAZI;

use crate::{Composer, file::FileRef};

pub(super) struct Plugin;

impl Plugin {
	pub(super) fn compose(lua: &Lua) -> mlua::Result<Value> {
		Composer::make(lua, 5, |lua, key| {
			match key {
				b"fetchers" => Plugin::fetchers(lua)?,
				b"spotter" => Plugin::spotter(lua)?,
				b"preloaders" => Plugin::preloaders(lua)?,
				b"previewer" => Plugin::previewer(lua)?,
				_ => return Ok(Value::Nil),
			}
			.into_lua(lua)
		})
	}

	fn fetchers(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|lua, (file, mime): (FileRef, mlua::String)| {
			lua.create_sequence_from(YAZI.plugin.fetchers(&file.url, &mime.to_str()?).map(Fetcher::new))
		})
	}

	fn spotter(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|_, (url, mime): (UrlRef, mlua::String)| {
			Ok(YAZI.plugin.spotter(&url, &mime.to_str()?).map(Spotter::new))
		})
	}

	fn preloaders(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|lua, (url, mime): (UrlRef, mlua::String)| {
			lua.create_sequence_from(YAZI.plugin.preloaders(&url, &mime.to_str()?).map(Preloader::new))
		})
	}

	fn previewer(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|_, (url, mime): (UrlRef, mlua::String)| {
			Ok(YAZI.plugin.previewer(&url, &mime.to_str()?).map(Previewer::new))
		})
	}
}

// --- Fetcher
struct Fetcher {
	inner: &'static yazi_config::plugin::Fetcher,

	v_cmd: Option<Value>,
}

impl Fetcher {
	pub fn new(inner: &'static yazi_config::plugin::Fetcher) -> Self { Self { inner, v_cmd: None } }
}

impl UserData for Fetcher {
	fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
		cached_field!(fields, cmd, |lua, me| lua.create_string(&me.inner.run.name));
	}
}

// --- Spotter
struct Spotter {
	inner: &'static yazi_config::plugin::Spotter,

	v_cmd: Option<Value>,
}

impl Spotter {
	pub fn new(inner: &'static yazi_config::plugin::Spotter) -> Self { Self { inner, v_cmd: None } }
}

impl UserData for Spotter {
	fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
		cached_field!(fields, cmd, |lua, me| lua.create_string(&me.inner.run.name));
	}
}

// --- Preloader
struct Preloader {
	inner: &'static yazi_config::plugin::Preloader,

	v_cmd: Option<Value>,
}

impl Preloader {
	pub fn new(inner: &'static yazi_config::plugin::Preloader) -> Self { Self { inner, v_cmd: None } }
}

impl UserData for Preloader {
	fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
		cached_field!(fields, cmd, |lua, me| lua.create_string(&me.inner.run.name));
	}
}

// --- Previewer
struct Previewer {
	inner: &'static yazi_config::plugin::Previewer,

	v_cmd: Option<Value>,
}

impl Previewer {
	pub fn new(inner: &'static yazi_config::plugin::Previewer) -> Self { Self { inner, v_cmd: None } }
}

impl UserData for Previewer {
	fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
		cached_field!(fields, cmd, |lua, me| lua.create_string(&me.inner.run.name));
	}
}
