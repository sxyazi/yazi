use mlua::{IntoLua, Lua, LuaSerdeExt, SerializeOptions, Value};
use yazi_binding::Url;
use yazi_boot::ARGS;
use yazi_config::YAZI;

use crate::Composer;

pub const OPTS: SerializeOptions =
	SerializeOptions::new().serialize_none_to_null(false).serialize_unit_to_null(false);

pub struct Runtime;

impl Runtime {
	pub fn compose(lua: &Lua) -> mlua::Result<Value> {
		Composer::make(lua, |lua, key| {
			match key {
				b"args" => Self::args(lua)?,
				b"term" => super::Term::compose(lua)?,
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
		Composer::make(lua, |lua, key| match key {
			b"entries" => lua.create_sequence_from(ARGS.entries.iter().map(Url::new))?.into_lua(lua),
			b"cwd_file" => ARGS.cwd_file.as_ref().map(Url::new).into_lua(lua),
			b"chooser_file" => ARGS.chooser_file.as_ref().map(Url::new).into_lua(lua),
			_ => Ok(Value::Nil),
		})
	}

	fn mgr(lua: &Lua) -> mlua::Result<Value> {
		Composer::make(lua, |lua, key| {
			let m = &YAZI.mgr;
			match key {
				b"ratio" => lua.to_value_with(&m.ratio, OPTS)?,

				b"sort_by" => lua.to_value_with(&m.sort_by, OPTS)?,
				b"sort_sensitive" => lua.to_value_with(&m.sort_sensitive, OPTS)?,
				b"sort_reverse" => lua.to_value_with(&m.sort_reverse, OPTS)?,
				b"sort_dir_first" => lua.to_value_with(&m.sort_dir_first, OPTS)?,
				b"sort_translit" => lua.to_value_with(&m.sort_translit, OPTS)?,

				b"linemode" => lua.to_value_with(&m.linemode, OPTS)?,
				b"show_hidden" => lua.to_value_with(&m.show_hidden, OPTS)?,
				b"show_symlink" => lua.to_value_with(&m.show_symlink, OPTS)?,
				b"scrolloff" => lua.to_value_with(&m.scrolloff, OPTS)?,
				b"mouse_events" => lua.to_value_with(&m.mouse_events, OPTS)?,
				b"title_format" => lua.to_value_with(&m.title_format, OPTS)?,
				_ => return Ok(Value::Nil),
			}
			.into_lua(lua)
		})
	}

	fn preview(lua: &Lua) -> mlua::Result<Value> {
		Composer::make(lua, |lua, key| {
			let p = &YAZI.preview;
			match key {
				b"wrap" => lua.to_value_with(&p.wrap, OPTS)?,
				b"tab_size" => lua.to_value_with(&p.tab_size, OPTS)?,
				b"max_width" => lua.to_value_with(&p.max_width, OPTS)?,
				b"max_height" => lua.to_value_with(&p.max_height, OPTS)?,

				b"cache_dir" => lua.to_value_with(&p.cache_dir, OPTS)?,

				b"image_delay" => lua.to_value_with(&p.image_delay, OPTS)?,
				b"image_filter" => lua.to_value_with(&p.image_filter, OPTS)?,
				b"image_quality" => lua.to_value_with(&p.image_quality, OPTS)?,

				b"ueberzug_scale" => lua.to_value_with(&p.ueberzug_scale, OPTS)?,
				b"ueberzug_offset" => lua.to_value_with(&p.ueberzug_offset, OPTS)?,
				_ => return Ok(Value::Nil),
			}
			.into_lua(lua)
		})
	}

	fn tasks(lua: &Lua) -> mlua::Result<Value> {
		Composer::make(lua, |lua, key| {
			let t = &YAZI.tasks;
			match key {
				b"micro_workers" => lua.to_value_with(&t.micro_workers, OPTS)?,
				b"macro_workers" => lua.to_value_with(&t.macro_workers, OPTS)?,
				b"bizarre_retry" => lua.to_value_with(&t.bizarre_retry, OPTS)?,

				b"image_alloc" => lua.to_value_with(&t.image_alloc, OPTS)?,
				b"image_bound" => lua.to_value_with(&t.image_bound, OPTS)?,

				b"suppress_preload" => lua.to_value_with(&t.suppress_preload, OPTS)?,
				_ => return Ok(Value::Nil),
			}
			.into_lua(lua)
		})
	}
}
