use mlua::{IntoLua, Lua, LuaSerdeExt, SerializeOptions, Value};
use yazi_adapter::EMULATOR;
use yazi_boot::ARGS;
use yazi_config::{MGR, PREVIEW, TASKS, THEME};

use crate::{Composer, url::Url};

pub const OPTS: SerializeOptions =
	SerializeOptions::new().serialize_none_to_null(false).serialize_unit_to_null(false);

pub struct Runtime<'a> {
	lua: &'a Lua,
}

impl<'a> Runtime<'a> {
	pub fn compose(lua: &Lua) -> mlua::Result<Value> {
		Composer::make(lua, 10, |lua, key| {
			match key {
				b"args" => Self::args(lua)?,
				b"term" => Self::term(lua)?,
				b"mgr" => Self::mgr(lua)?,
				b"plugin" => super::Plugin::compose(lua)?,
				b"preview" => Self::preview(lua)?,
				b"tasks" => Self::tasks(lua)?,
				_ => return Ok(Value::Nil),
			}
			.into_lua(lua)
		})
	}

	fn args(lua: &Lua) -> mlua::Result<Value> {
		Composer::make(lua, 5, |lua, key| {
			match key {
				b"chooser_file" => ARGS.chooser_file.as_ref().map(Url::from).into_lua(lua)?,
				b"cwd_file" => ARGS.cwd_file.as_ref().map(Url::from).into_lua(lua)?,
				_ => return Ok(Value::Nil),
			}
			.into_lua(lua)
		})
	}

	fn term(lua: &Lua) -> mlua::Result<Value> {
		lua.create_table_from([("light", EMULATOR.get().light)])?.into_lua(lua)
	}

	fn mgr(lua: &Lua) -> mlua::Result<Value> {
		Composer::make(lua, 15, |lua, key| {
			match key {
				b"ratio" => lua.to_value_with(&MGR.ratio, OPTS)?,

				b"sort_by" => lua.to_value_with(&MGR.sort_by, OPTS)?,
				b"sort_sensitive" => lua.to_value_with(&MGR.sort_sensitive, OPTS)?,
				b"sort_reverse" => lua.to_value_with(&MGR.sort_reverse, OPTS)?,
				b"sort_dir_first" => lua.to_value_with(&MGR.sort_dir_first, OPTS)?,
				b"sort_translit" => lua.to_value_with(&MGR.sort_translit, OPTS)?,

				b"linemode" => lua.to_value_with(&MGR.linemode, OPTS)?,
				b"show_hidden" => lua.to_value_with(&MGR.show_hidden, OPTS)?,
				b"show_symlink" => lua.to_value_with(&MGR.show_symlink, OPTS)?,
				b"scrolloff" => lua.to_value_with(&MGR.scrolloff, OPTS)?,
				b"mouse_events" => lua.to_value_with(&MGR.mouse_events, OPTS)?,
				b"title_format" => lua.to_value_with(&MGR.title_format, OPTS)?,
				_ => return Ok(Value::Nil),
			}
			.into_lua(lua)
		})
	}

	fn preview(lua: &Lua) -> mlua::Result<Value> {
		Composer::make(lua, 15, |lua, key| {
			match key {
				b"wrap" => lua.to_value_with(&PREVIEW.wrap, OPTS)?,
				b"tab_size" => lua.to_value_with(&PREVIEW.tab_size, OPTS)?,
				b"max_width" => lua.to_value_with(&PREVIEW.max_width, OPTS)?,
				b"max_height" => lua.to_value_with(&PREVIEW.max_height, OPTS)?,

				b"cache_dir" => lua.to_value_with(&PREVIEW.cache_dir, OPTS)?,

				b"image_delay" => lua.to_value_with(&PREVIEW.image_delay, OPTS)?,
				b"image_filter" => lua.to_value_with(&PREVIEW.image_filter, OPTS)?,
				b"image_quality" => lua.to_value_with(&PREVIEW.image_quality, OPTS)?,
				b"sixel_fraction" => lua.to_value_with(&PREVIEW.sixel_fraction, OPTS)?,

				b"ueberzug_scale" => lua.to_value_with(&PREVIEW.ueberzug_scale, OPTS)?,
				b"ueberzug_offset" => lua.to_value_with(&PREVIEW.ueberzug_offset, OPTS)?,
				_ => return Ok(Value::Nil),
			}
			.into_lua(lua)
		})
	}

	fn tasks(lua: &Lua) -> mlua::Result<Value> {
		Composer::make(lua, 10, |lua, key| {
			match key {
				b"micro_workers" => lua.to_value_with(&TASKS.micro_workers, OPTS)?,
				b"macro_workers" => lua.to_value_with(&TASKS.macro_workers, OPTS)?,
				b"bizarre_retry" => lua.to_value_with(&TASKS.bizarre_retry, OPTS)?,

				b"image_alloc" => lua.to_value_with(&TASKS.image_alloc, OPTS)?,
				b"image_bound" => lua.to_value_with(&TASKS.image_bound, OPTS)?,

				b"suppress_preload" => lua.to_value_with(&TASKS.suppress_preload, OPTS)?,
				_ => return Ok(Value::Nil),
			}
			.into_lua(lua)
		})
	}

	pub fn new(lua: &'a Lua) -> Self { Self { lua } }

	// TODO: remove this
	pub fn install_manager(self) -> mlua::Result<Self> {
		self.lua.globals().raw_set("MANAGER", self.lua.to_value_with(&*MGR, OPTS)?)?;
		Ok(self)
	}

	// TODO: remove this
	pub fn install_theme(self) -> mlua::Result<Self> {
		self.lua.globals().raw_set("THEME", self.lua.to_value_with(&*THEME, OPTS)?)?;
		Ok(self)
	}

	// TODO: remove this
	pub fn install_preview(self) -> mlua::Result<Self> {
		self.lua.globals().raw_set("PREVIEW", self.lua.to_value_with(&*PREVIEW, OPTS)?)?;
		Ok(self)
	}
}
