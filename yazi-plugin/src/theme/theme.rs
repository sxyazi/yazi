use mlua::{IntoLua, Lua, Value};
use yazi_binding::{Composer, ComposerGet, ComposerSet, Style, Url, deprecate};
use yazi_config::THEME;

pub fn compose() -> Composer<ComposerGet, ComposerSet> {
	fn get(lua: &Lua, key: &[u8]) -> mlua::Result<Value> {
		match key {
			b"app" => app(),
			b"mgr" => mgr(),
			b"tabs" => tabs(),
			b"mode" => mode(),
			b"indicator" => indicator(),
			b"status" => status(),
			b"which" => which(),
			b"confirm" => confirm(),
			b"spot" => spot(),
			b"notify" => notify(),
			b"pick" => pick(),
			b"input" => input(),
			b"cmp" => cmp(),
			b"tasks" => tasks(),
			b"help" => help(),
			_ => return Ok(Value::Nil),
		}
		.into_lua(lua)
	}

	fn set(_: &Lua, _: &[u8], value: Value) -> mlua::Result<Value> { Ok(value) }

	Composer::new(get, set)
}

fn app() -> Composer<ComposerGet, ComposerSet> {
	fn get(lua: &Lua, key: &[u8]) -> mlua::Result<Value> {
		let a = &THEME.app;
		match key {
			b"overall" => Style::from(a.overall).into_lua(lua),

			_ => Ok(Value::Nil),
		}
	}

	fn set(_: &Lua, _: &[u8], value: Value) -> mlua::Result<Value> { Ok(value) }

	Composer::new(get, set)
}

fn mgr() -> Composer<ComposerGet, ComposerSet> {
	fn get(lua: &Lua, key: &[u8]) -> mlua::Result<Value> {
		let m = &THEME.mgr;
		match key {
			b"cwd" => Style::from(m.cwd).into_lua(lua),

			b"hovered" => {
				deprecate!(
					lua,
					"`th.mgr.hovered` is deprecated, use `th.indicator.current` instead, in your {}\nSee #3419 for more details: https://github.com/sxyazi/yazi/pull/3419"
				);
				Style::from(THEME.indicator.current).into_lua(lua)
			}
			b"preview_hovered" => {
				deprecate!(
					lua,
					"`th.mgr.preview_hovered` is deprecated, use `th.indicator.preview` instead, in your {}\nSee #3419 for more details: https://github.com/sxyazi/yazi/pull/3419"
				);
				Style::from(THEME.indicator.preview).into_lua(lua)
			}

			b"find_keyword" => Style::from(m.find_keyword).into_lua(lua),
			b"find_position" => Style::from(m.find_position).into_lua(lua),

			b"symlink_target" => Style::from(m.symlink_target).into_lua(lua),

			b"marker_copied" => Style::from(m.marker_copied).into_lua(lua),
			b"marker_cut" => Style::from(m.marker_cut).into_lua(lua),
			b"marker_marked" => Style::from(m.marker_marked).into_lua(lua),
			b"marker_selected" => Style::from(m.marker_selected).into_lua(lua),

			b"count_copied" => Style::from(m.count_copied).into_lua(lua),
			b"count_cut" => Style::from(m.count_cut).into_lua(lua),
			b"count_selected" => Style::from(m.count_selected).into_lua(lua),

			b"border_symbol" => lua.create_string(&m.border_symbol)?.into_lua(lua),
			b"border_style" => Style::from(m.border_style).into_lua(lua),

			b"syntect_theme" => Url::new(&m.syntect_theme).into_lua(lua),
			_ => Ok(Value::Nil),
		}
	}

	fn set(_: &Lua, _: &[u8], value: Value) -> mlua::Result<Value> { Ok(value) }

	Composer::new(get, set)
}

fn tabs() -> Composer<ComposerGet, ComposerSet> {
	fn get(lua: &Lua, key: &[u8]) -> mlua::Result<Value> {
		let t = &THEME.tabs;
		match key {
			b"active" => Style::from(t.active).into_lua(lua),
			b"inactive" => Style::from(t.inactive).into_lua(lua),

			b"sep_inner" => lua
				.create_table_from([
					("open", lua.create_string(&t.sep_inner.open)?),
					("close", lua.create_string(&t.sep_inner.close)?),
				])?
				.into_lua(lua),
			b"sep_outer" => lua
				.create_table_from([
					("open", lua.create_string(&t.sep_outer.open)?),
					("close", lua.create_string(&t.sep_outer.close)?),
				])?
				.into_lua(lua),

			_ => Ok(Value::Nil),
		}
	}

	fn set(_: &Lua, _: &[u8], value: Value) -> mlua::Result<Value> { Ok(value) }

	Composer::new(get, set)
}

fn mode() -> Composer<ComposerGet, ComposerSet> {
	fn get(lua: &Lua, key: &[u8]) -> mlua::Result<Value> {
		let t = &THEME.mode;
		match key {
			b"normal_main" => Style::from(t.normal_main).into_lua(lua),
			b"normal_alt" => Style::from(t.normal_alt).into_lua(lua),

			b"select_main" => Style::from(t.select_main).into_lua(lua),
			b"select_alt" => Style::from(t.select_alt).into_lua(lua),

			b"unset_main" => Style::from(t.unset_main).into_lua(lua),
			b"unset_alt" => Style::from(t.unset_alt).into_lua(lua),

			_ => Ok(Value::Nil),
		}
	}

	fn set(_: &Lua, _: &[u8], value: Value) -> mlua::Result<Value> { Ok(value) }

	Composer::new(get, set)
}

fn indicator() -> Composer<ComposerGet, ComposerSet> {
	fn get(lua: &Lua, key: &[u8]) -> mlua::Result<Value> {
		let t = &THEME.indicator;
		match key {
			b"parent" => Style::from(t.parent).into_lua(lua),
			b"current" => Style::from(t.current).into_lua(lua),
			b"preview" => Style::from(t.preview).into_lua(lua),
			b"padding" => lua
				.create_table_from([
					("open", lua.create_string(&t.padding.open)?),
					("close", lua.create_string(&t.padding.close)?),
				])?
				.into_lua(lua),

			_ => Ok(Value::Nil),
		}
	}

	fn set(_: &Lua, _: &[u8], value: Value) -> mlua::Result<Value> { Ok(value) }

	Composer::new(get, set)
}

fn status() -> Composer<ComposerGet, ComposerSet> {
	fn get(lua: &Lua, key: &[u8]) -> mlua::Result<Value> {
		let t = &THEME.status;
		match key {
			b"overall" => Style::from(t.overall).into_lua(lua),
			b"sep_left" => lua
				.create_table_from([
					("open", lua.create_string(&t.sep_left.open)?),
					("close", lua.create_string(&t.sep_left.close)?),
				])?
				.into_lua(lua),
			b"sep_right" => lua
				.create_table_from([
					("open", lua.create_string(&t.sep_right.open)?),
					("close", lua.create_string(&t.sep_right.close)?),
				])?
				.into_lua(lua),

			b"perm_sep" => Style::from(t.perm_sep).into_lua(lua),
			b"perm_type" => Style::from(t.perm_type).into_lua(lua),
			b"perm_read" => Style::from(t.perm_read).into_lua(lua),
			b"perm_write" => Style::from(t.perm_write).into_lua(lua),
			b"perm_exec" => Style::from(t.perm_exec).into_lua(lua),

			b"progress_label" => Style::from(t.progress_label).into_lua(lua),
			b"progress_normal" => Style::from(t.progress_normal).into_lua(lua),
			b"progress_error" => Style::from(t.progress_error).into_lua(lua),

			_ => Ok(Value::Nil),
		}
	}

	fn set(_: &Lua, _: &[u8], value: Value) -> mlua::Result<Value> { Ok(value) }

	Composer::new(get, set)
}

fn which() -> Composer<ComposerGet, ComposerSet> {
	fn get(lua: &Lua, key: &[u8]) -> mlua::Result<Value> {
		let t = &THEME.which;
		match key {
			b"cols" => t.cols.into_lua(lua),
			b"mask" => Style::from(t.mask).into_lua(lua),
			b"cand" => Style::from(t.cand).into_lua(lua),
			b"rest" => Style::from(t.rest).into_lua(lua),
			b"desc" => Style::from(t.desc).into_lua(lua),

			b"separator" => lua.create_string(&t.separator)?.into_lua(lua),
			b"separator_style" => Style::from(t.separator_style).into_lua(lua),

			_ => Ok(Value::Nil),
		}
	}

	fn set(_: &Lua, _: &[u8], value: Value) -> mlua::Result<Value> { Ok(value) }

	Composer::new(get, set)
}

fn confirm() -> Composer<ComposerGet, ComposerSet> {
	fn get(lua: &Lua, key: &[u8]) -> mlua::Result<Value> {
		let t = &THEME.confirm;
		match key {
			b"border" => Style::from(t.border).into_lua(lua),
			b"title" => Style::from(t.title).into_lua(lua),
			b"body" => Style::from(t.body).into_lua(lua),
			b"list" => Style::from(t.list).into_lua(lua),

			b"btn_yes" => Style::from(t.btn_yes).into_lua(lua),
			b"btn_no" => Style::from(t.btn_no).into_lua(lua),
			b"btn_labels" => lua
				.create_sequence_from([
					lua.create_string(&t.btn_labels[0])?,
					lua.create_string(&t.btn_labels[1])?,
				])?
				.into_lua(lua),

			_ => Ok(Value::Nil),
		}
	}

	fn set(_: &Lua, _: &[u8], value: Value) -> mlua::Result<Value> { Ok(value) }

	Composer::new(get, set)
}

fn spot() -> Composer<ComposerGet, ComposerSet> {
	fn get(lua: &Lua, key: &[u8]) -> mlua::Result<Value> {
		let t = &THEME.spot;
		match key {
			b"border" => Style::from(t.border).into_lua(lua),
			b"title" => Style::from(t.title).into_lua(lua),

			b"tbl_col" => Style::from(t.tbl_col).into_lua(lua),
			b"tbl_cell" => Style::from(t.tbl_cell).into_lua(lua),

			_ => Ok(Value::Nil),
		}
	}

	fn set(_: &Lua, _: &[u8], value: Value) -> mlua::Result<Value> { Ok(value) }

	Composer::new(get, set)
}

fn notify() -> Composer<ComposerGet, ComposerSet> {
	fn get(lua: &Lua, key: &[u8]) -> mlua::Result<Value> {
		let t = &THEME.notify;
		match key {
			b"title_info" => Style::from(t.title_info).into_lua(lua),
			b"title_warn" => Style::from(t.title_warn).into_lua(lua),
			b"title_error" => Style::from(t.title_error).into_lua(lua),

			b"icon_info" => lua.create_string(&t.icon_info)?.into_lua(lua),
			b"icon_warn" => lua.create_string(&t.icon_warn)?.into_lua(lua),
			b"icon_error" => lua.create_string(&t.icon_error)?.into_lua(lua),

			_ => Ok(Value::Nil),
		}
	}

	fn set(_: &Lua, _: &[u8], value: Value) -> mlua::Result<Value> { Ok(value) }

	Composer::new(get, set)
}

fn pick() -> Composer<ComposerGet, ComposerSet> {
	fn get(lua: &Lua, key: &[u8]) -> mlua::Result<Value> {
		let t = &THEME.pick;
		match key {
			b"border" => Style::from(t.border).into_lua(lua),
			b"active" => Style::from(t.active).into_lua(lua),
			b"inactive" => Style::from(t.inactive).into_lua(lua),

			_ => Ok(Value::Nil),
		}
	}

	fn set(_: &Lua, _: &[u8], value: Value) -> mlua::Result<Value> { Ok(value) }

	Composer::new(get, set)
}

fn input() -> Composer<ComposerGet, ComposerSet> {
	fn get(lua: &Lua, key: &[u8]) -> mlua::Result<Value> {
		let t = &THEME.input;
		match key {
			b"border" => Style::from(t.border).into_lua(lua),
			b"title" => Style::from(t.title).into_lua(lua),
			b"value" => Style::from(t.value).into_lua(lua),
			b"selected" => Style::from(t.selected).into_lua(lua),

			_ => Ok(Value::Nil),
		}
	}

	fn set(_: &Lua, _: &[u8], value: Value) -> mlua::Result<Value> { Ok(value) }

	Composer::new(get, set)
}

fn cmp() -> Composer<ComposerGet, ComposerSet> {
	fn get(lua: &Lua, key: &[u8]) -> mlua::Result<Value> {
		let t = &THEME.cmp;
		match key {
			b"border" => Style::from(t.border).into_lua(lua),
			b"active" => Style::from(t.active).into_lua(lua),
			b"inactive" => Style::from(t.inactive).into_lua(lua),

			b"icon_file" => lua.create_string(&t.icon_file)?.into_lua(lua),
			b"icon_folder" => lua.create_string(&t.icon_folder)?.into_lua(lua),
			b"icon_command" => lua.create_string(&t.icon_command)?.into_lua(lua),

			_ => Ok(Value::Nil),
		}
	}

	fn set(_: &Lua, _: &[u8], value: Value) -> mlua::Result<Value> { Ok(value) }

	Composer::new(get, set)
}

fn tasks() -> Composer<ComposerGet, ComposerSet> {
	fn get(lua: &Lua, key: &[u8]) -> mlua::Result<Value> {
		let t = &THEME.tasks;
		match key {
			b"border" => Style::from(t.border).into_lua(lua),
			b"title" => Style::from(t.title).into_lua(lua),
			b"hovered" => Style::from(t.hovered).into_lua(lua),

			_ => Ok(Value::Nil),
		}
	}

	fn set(_: &Lua, _: &[u8], value: Value) -> mlua::Result<Value> { Ok(value) }

	Composer::new(get, set)
}

fn help() -> Composer<ComposerGet, ComposerSet> {
	fn get(lua: &Lua, key: &[u8]) -> mlua::Result<Value> {
		let t = &THEME.help;
		match key {
			b"on" => Style::from(t.on).into_lua(lua),
			b"run" => Style::from(t.run).into_lua(lua),
			b"desc" => Style::from(t.desc).into_lua(lua),

			b"hovered" => Style::from(t.hovered).into_lua(lua),
			b"footer" => Style::from(t.footer).into_lua(lua),

			_ => Ok(Value::Nil),
		}
	}

	fn set(_: &Lua, _: &[u8], value: Value) -> mlua::Result<Value> { Ok(value) }

	Composer::new(get, set)
}
