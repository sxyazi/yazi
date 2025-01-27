use mlua::{Function, Lua, UserData};
use yazi_config::PLUGIN;

use crate::{file::FileRef, url::UrlRef};

pub(super) struct Plugin;

impl Plugin {
	pub(super) fn fetchers(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|lua, (file, mime): (FileRef, mlua::String)| {
			lua.create_sequence_from(PLUGIN.fetchers(&file.url, &mime.to_str()?).map(Fetcher))
		})
	}

	pub(super) fn spotter(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|_, (url, mime): (UrlRef, mlua::String)| {
			Ok(PLUGIN.spotter(&url, &mime.to_str()?).map(Spotter))
		})
	}

	pub(super) fn preloaders(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|lua, (url, mime): (UrlRef, mlua::String)| {
			lua.create_sequence_from(PLUGIN.preloaders(&url, &mime.to_str()?).map(Preloader))
		})
	}

	pub(super) fn previewer(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|_, (url, mime): (UrlRef, mlua::String)| {
			Ok(PLUGIN.previewer(&url, &mime.to_str()?).map(Previewer))
		})
	}
}

// --- Fetcher
struct Fetcher(&'static yazi_config::plugin::Fetcher);

impl UserData for Fetcher {
	fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("cmd", |lua, me| lua.create_string(&me.0.run.name));
	}
}

// --- Spotter
struct Spotter(&'static yazi_config::plugin::Spotter);

impl UserData for Spotter {
	fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("cmd", |lua, me| lua.create_string(&me.0.run.name));
	}
}

// --- Preloader
struct Preloader(&'static yazi_config::plugin::Preloader);

impl UserData for Preloader {
	fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("cmd", |lua, me| lua.create_string(&me.0.run.name));
	}
}

// --- Previewer
struct Previewer(&'static yazi_config::plugin::Previewer);

impl UserData for Previewer {
	fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("cmd", |lua, me| lua.create_string(&me.0.run.name));
	}
}
