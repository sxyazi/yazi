use mlua::{IntoLua, Lua, LuaSerdeExt, Value};
use yazi_binding::{Composer, ComposerGet, ComposerSet, SER_OPT, Url};
use yazi_boot::ARGS;
use yazi_config::YAZI;

pub fn compose() -> Composer<ComposerGet, ComposerSet> {
	fn get(lua: &Lua, key: &[u8]) -> mlua::Result<Value> {
		match key {
			b"args" => args().into_lua(lua)?,
			b"term" => super::term().into_lua(lua)?,
			b"mgr" => mgr().into_lua(lua)?,
			b"plugin" => super::plugin().into_lua(lua)?,
			b"preview" => preview().into_lua(lua)?,
			b"tasks" => tasks().into_lua(lua)?,
			_ => return Ok(Value::Nil),
		}
		.into_lua(lua)
	}

	fn set(_: &Lua, _: &[u8], value: Value) -> mlua::Result<Value> { Ok(value) }

	Composer::new(get, set)
}

fn args() -> Composer<ComposerGet, ComposerSet> {
	fn get(lua: &Lua, key: &[u8]) -> mlua::Result<Value> {
		match key {
			b"entries" => lua.create_sequence_from(ARGS.entries.iter().map(Url::new))?.into_lua(lua),
			b"cwd_file" => ARGS.cwd_file.as_ref().map(Url::new).into_lua(lua),
			b"chooser_file" => ARGS.chooser_file.as_ref().map(Url::new).into_lua(lua),
			_ => Ok(Value::Nil),
		}
	}

	fn set(_: &Lua, _: &[u8], value: Value) -> mlua::Result<Value> { Ok(value) }

	Composer::new(get, set)
}

fn mgr() -> Composer<ComposerGet, ComposerSet> {
	fn get(lua: &Lua, key: &[u8]) -> mlua::Result<Value> {
		let m = &YAZI.mgr;
		match key {
			b"ratio" => lua.to_value_with(&m.ratio, SER_OPT)?,

			b"sort_by" => lua.to_value_with(&m.sort_by, SER_OPT)?,
			b"sort_sensitive" => lua.to_value_with(&m.sort_sensitive, SER_OPT)?,
			b"sort_reverse" => lua.to_value_with(&m.sort_reverse, SER_OPT)?,
			b"sort_dir_first" => lua.to_value_with(&m.sort_dir_first, SER_OPT)?,
			b"sort_translit" => lua.to_value_with(&m.sort_translit, SER_OPT)?,

			b"linemode" => lua.to_value_with(&m.linemode, SER_OPT)?,
			b"show_hidden" => lua.to_value_with(&m.show_hidden, SER_OPT)?,
			b"show_symlink" => lua.to_value_with(&m.show_symlink, SER_OPT)?,
			b"scrolloff" => lua.to_value_with(&m.scrolloff, SER_OPT)?,
			b"mouse_events" => lua.to_value_with(&m.mouse_events, SER_OPT)?,
			b"title_format" => lua.to_value_with(&m.title_format, SER_OPT)?,
			_ => return Ok(Value::Nil),
		}
		.into_lua(lua)
	}

	fn set(lua: &Lua, key: &[u8], value: Value) -> mlua::Result<Value> {
		let m = &YAZI.mgr;
		Ok(match key {
			b"ratio" => {
				m.ratio.set(lua.from_value(value)?);
				Value::Nil
			}
			_ => value,
		})
	}

	Composer::new(get, set)
}

fn preview() -> Composer<ComposerGet, ComposerSet> {
	fn get(lua: &Lua, key: &[u8]) -> mlua::Result<Value> {
		let p = &YAZI.preview;
		match key {
			b"wrap" => lua.to_value_with(&p.wrap, SER_OPT)?,
			b"tab_size" => lua.to_value_with(&p.tab_size, SER_OPT)?,
			b"max_width" => lua.to_value_with(&p.max_width, SER_OPT)?,
			b"max_height" => lua.to_value_with(&p.max_height, SER_OPT)?,

			b"cache_dir" => lua.to_value_with(&p.cache_dir, SER_OPT)?,

			b"image_delay" => lua.to_value_with(&p.image_delay, SER_OPT)?,
			b"image_filter" => lua.to_value_with(&p.image_filter, SER_OPT)?,
			b"image_quality" => lua.to_value_with(&p.image_quality, SER_OPT)?,

			b"ueberzug_scale" => lua.to_value_with(&p.ueberzug_scale, SER_OPT)?,
			b"ueberzug_offset" => lua.to_value_with(&p.ueberzug_offset, SER_OPT)?,
			_ => return Ok(Value::Nil),
		}
		.into_lua(lua)
	}

	fn set(_: &Lua, _: &[u8], value: Value) -> mlua::Result<Value> { Ok(value) }

	Composer::new(get, set)
}

fn tasks() -> Composer<ComposerGet, ComposerSet> {
	fn get(lua: &Lua, key: &[u8]) -> mlua::Result<Value> {
		let t = &YAZI.tasks;
		match key {
			b"micro_workers" => lua.to_value_with(&t.micro_workers, SER_OPT)?,
			b"macro_workers" => lua.to_value_with(&t.macro_workers, SER_OPT)?,
			b"bizarre_retry" => lua.to_value_with(&t.bizarre_retry, SER_OPT)?,

			b"image_alloc" => lua.to_value_with(&t.image_alloc, SER_OPT)?,
			b"image_bound" => lua.to_value_with(&t.image_bound, SER_OPT)?,

			b"suppress_preload" => lua.to_value_with(&t.suppress_preload, SER_OPT)?,
			_ => return Ok(Value::Nil),
		}
		.into_lua(lua)
	}

	fn set(_: &Lua, _: &[u8], value: Value) -> mlua::Result<Value> { Ok(value) }

	Composer::new(get, set)
}
