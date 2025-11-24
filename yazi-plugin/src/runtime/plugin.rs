use mlua::{Function, IntoLua, Lua, UserData, Value};
use yazi_binding::{Composer, ComposerGet, ComposerSet, FileRef, cached_field};
use yazi_config::YAZI;

pub(super) fn plugin() -> Composer<ComposerGet, ComposerSet> {
	fn get(lua: &Lua, key: &[u8]) -> mlua::Result<Value> {
		match key {
			b"fetchers" => fetchers(lua)?,
			b"spotter" => spotter(lua)?,
			b"preloaders" => preloaders(lua)?,
			b"previewer" => previewer(lua)?,
			_ => return Ok(Value::Nil),
		}
		.into_lua(lua)
	}

	fn set(_: &Lua, _: &[u8], value: Value) -> mlua::Result<Value> { Ok(value) }

	Composer::new(get, set)
}

fn fetchers(lua: &Lua) -> mlua::Result<Function> {
	lua.create_function(|lua, (file, mime): (FileRef, mlua::String)| {
		lua.create_sequence_from(YAZI.plugin.fetchers(&file, &mime.to_str()?).map(Fetcher::new))
	})
}

fn spotter(lua: &Lua) -> mlua::Result<Function> {
	lua.create_function(|_, (file, mime): (FileRef, mlua::String)| {
		Ok(YAZI.plugin.spotter(&file, &mime.to_str()?).map(Spotter::new))
	})
}

fn preloaders(lua: &Lua) -> mlua::Result<Function> {
	lua.create_function(|lua, (file, mime): (FileRef, mlua::String)| {
		lua.create_sequence_from(YAZI.plugin.preloaders(&file, &mime.to_str()?).map(Preloader::new))
	})
}

fn previewer(lua: &Lua) -> mlua::Result<Function> {
	lua.create_function(|_, (file, mime): (FileRef, mlua::String)| {
		Ok(YAZI.plugin.previewer(&file, &mime.to_str()?).map(Previewer::new))
	})
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
		cached_field!(fields, cmd, |lua, me| lua.create_string(&*me.inner.run.name));
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
		cached_field!(fields, cmd, |lua, me| lua.create_string(&*me.inner.run.name));
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
		cached_field!(fields, cmd, |lua, me| lua.create_string(&*me.inner.run.name));
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
		cached_field!(fields, cmd, |lua, me| lua.create_string(&*me.inner.run.name));
	}
}
