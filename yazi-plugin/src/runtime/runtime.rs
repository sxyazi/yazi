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
			b"sort_sensitive" => m.sort_sensitive.get().into_lua(lua)?,
			b"sort_reverse" => m.sort_reverse.get().into_lua(lua)?,
			b"sort_dir_first" => m.sort_dir_first.get().into_lua(lua)?,
			b"sort_translit" => m.sort_translit.get().into_lua(lua)?,

			b"linemode" => lua.create_string(&m.linemode)?.into_lua(lua)?,
			b"show_hidden" => m.show_hidden.get().into_lua(lua)?,
			b"show_symlink" => m.show_symlink.get().into_lua(lua)?,
			b"scrolloff" => m.scrolloff.get().into_lua(lua)?,
			b"mouse_events" => lua.to_value_with(&m.mouse_events, SER_OPT)?,
			b"title_format" => lua.create_string(&m.title_format)?.into_lua(lua)?,
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
			b"tab_size" => p.tab_size.into_lua(lua)?,
			b"max_width" => p.max_width.into_lua(lua)?,
			b"max_height" => p.max_height.into_lua(lua)?,

			b"cache_dir" => lua.to_value_with(&p.cache_dir, SER_OPT)?,

			b"image_delay" => p.image_delay.into_lua(lua)?,
			b"image_filter" => lua.create_string(&p.image_filter)?.into_lua(lua)?,
			b"image_quality" => p.image_quality.into_lua(lua)?,

			b"ueberzug_scale" => p.ueberzug_scale.into_lua(lua)?,
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
			b"file_workers" => t.file_workers.into_lua(lua)?,
			b"plugin_workers" => t.plugin_workers.into_lua(lua)?,
			b"fetch_workers" => t.fetch_workers.into_lua(lua)?,
			b"preload_workers" => t.preload_workers.into_lua(lua)?,
			b"process_workers" => t.process_workers.into_lua(lua)?,
			b"bizarre_retry" => t.bizarre_retry.into_lua(lua)?,

			b"image_alloc" => t.image_alloc.into_lua(lua)?,
			b"image_bound" => lua.to_value_with(&t.image_bound, SER_OPT)?,

			b"suppress_preload" => t.suppress_preload.into_lua(lua)?,
			_ => return Ok(Value::Nil),
		}
		.into_lua(lua)
	}

	fn set(_: &Lua, _: &[u8], value: Value) -> mlua::Result<Value> { Ok(value) }

	Composer::new(get, set)
}
