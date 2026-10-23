use mlua::{IntoLua, Lua, Value};
use yazi_binding::{Composer, ComposerGet};
use yazi_config::{THEME, theme::CustomSectionArc};
use yazi_shared::url::UrlBuf;

use crate::{LUA, theme::icon};

pub(crate) fn compose() -> Composer<ComposerGet> {
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
			b"icon" => icon(),
			_ => return custom(lua, key),
		}
		.into_lua(lua)
	}

	Composer::new(get)
}

pub fn reset() -> mlua::Result<()> { LUA.globals().raw_set("th", compose()) }

fn app() -> Composer<ComposerGet> {
	fn get(lua: &Lua, key: &[u8]) -> mlua::Result<Value> {
		let a = &THEME.app;
		match key {
			b"overall" => a.overall.into_lua(lua),

			_ => Ok(Value::Nil),
		}
	}

	Composer::new(get)
}

fn mgr() -> Composer<ComposerGet> {
	fn get(lua: &Lua, key: &[u8]) -> mlua::Result<Value> {
		let m = &THEME.mgr;
		match key {
			b"cwd" => m.cwd.into_lua(lua),

			b"find_keyword" => m.find_keyword.into_lua(lua),
			b"find_position" => m.find_position.into_lua(lua),

			b"symlink_target" => m.symlink_target.into_lua(lua),

			b"marker_copied" => m.marker_copied.into_lua(lua),
			b"marker_cut" => m.marker_cut.into_lua(lua),
			b"marker_marked" => m.marker_marked.into_lua(lua),
			b"marker_selected" => m.marker_selected.into_lua(lua),
			b"marker_symbol" => lua.create_string(&**m.marker_symbol.load())?.into_lua(lua),

			b"count_copied" => m.count_copied.into_lua(lua),
			b"count_cut" => m.count_cut.into_lua(lua),
			b"count_selected" => m.count_selected.into_lua(lua),

			b"border_symbol" => lua.create_string(&**m.border_symbol.load())?.into_lua(lua),
			b"border_style" => m.border_style.into_lua(lua),

			b"syntect_theme" => UrlBuf::from(&**m.syntect_theme.load()).into_lua(lua),
			_ => Ok(Value::Nil),
		}
	}

	Composer::new(get)
}

fn tabs() -> Composer<ComposerGet> {
	fn get(lua: &Lua, key: &[u8]) -> mlua::Result<Value> {
		let t = &THEME.tabs;
		match key {
			b"active" => t.active.into_lua(lua),
			b"inactive" => t.inactive.into_lua(lua),

			b"sep_inner" => lua
				.create_table_from([
					("open", lua.create_string(&t.sep_inner.load().open)?),
					("close", lua.create_string(&t.sep_inner.load().close)?),
				])?
				.into_lua(lua),
			b"sep_outer" => lua
				.create_table_from([
					("open", lua.create_string(&t.sep_outer.load().open)?),
					("close", lua.create_string(&t.sep_outer.load().close)?),
				])?
				.into_lua(lua),

			_ => Ok(Value::Nil),
		}
	}

	Composer::new(get)
}

fn mode() -> Composer<ComposerGet> {
	fn get(lua: &Lua, key: &[u8]) -> mlua::Result<Value> {
		let t = &THEME.mode;
		match key {
			b"normal_main" => t.normal_main.into_lua(lua),
			b"normal_alt" => t.normal_alt.into_lua(lua),

			b"select_main" => t.select_main.into_lua(lua),
			b"select_alt" => t.select_alt.into_lua(lua),

			b"unset_main" => t.unset_main.into_lua(lua),
			b"unset_alt" => t.unset_alt.into_lua(lua),

			_ => Ok(Value::Nil),
		}
	}

	Composer::new(get)
}

fn indicator() -> Composer<ComposerGet> {
	fn get(lua: &Lua, key: &[u8]) -> mlua::Result<Value> {
		let t = &THEME.indicator;
		match key {
			b"parent" => t.parent.into_lua(lua),
			b"current" => t.current.into_lua(lua),
			b"preview" => t.preview.into_lua(lua),
			b"padding" => lua
				.create_table_from([
					("open", lua.create_string(&t.padding.load().open)?),
					("close", lua.create_string(&t.padding.load().close)?),
				])?
				.into_lua(lua),

			_ => Ok(Value::Nil),
		}
	}

	Composer::new(get)
}

fn status() -> Composer<ComposerGet> {
	fn get(lua: &Lua, key: &[u8]) -> mlua::Result<Value> {
		let t = &THEME.status;
		match key {
			b"overall" => t.overall.into_lua(lua),
			b"sep_left" => lua
				.create_table_from([
					("open", lua.create_string(&t.sep_left.load().open)?),
					("close", lua.create_string(&t.sep_left.load().close)?),
				])?
				.into_lua(lua),
			b"sep_right" => lua
				.create_table_from([
					("open", lua.create_string(&t.sep_right.load().open)?),
					("close", lua.create_string(&t.sep_right.load().close)?),
				])?
				.into_lua(lua),

			b"perm_sep" => t.perm_sep.into_lua(lua),
			b"perm_type" => t.perm_type.into_lua(lua),
			b"perm_read" => t.perm_read.into_lua(lua),
			b"perm_write" => t.perm_write.into_lua(lua),
			b"perm_exec" => t.perm_exec.into_lua(lua),

			b"progress_label" => t.progress_label.into_lua(lua),
			b"progress_normal" => t.progress_normal.into_lua(lua),
			b"progress_error" => t.progress_error.into_lua(lua),

			_ => Ok(Value::Nil),
		}
	}

	Composer::new(get)
}

fn which() -> Composer<ComposerGet> {
	fn get(lua: &Lua, key: &[u8]) -> mlua::Result<Value> {
		let t = &THEME.which;
		match key {
			b"border" => t.border.into_lua(lua),
			b"border_set" => t.border_set.load_full().into_lua(lua),
			b"cols" => t.cols.into_lua(lua),
			b"mask" => t.mask.into_lua(lua),
			b"cand" => t.cand.into_lua(lua),
			b"rest" => t.rest.into_lua(lua),
			b"desc" => t.desc.into_lua(lua),

			b"separator" => lua.create_string(&**t.separator.load())?.into_lua(lua),
			b"separator_style" => t.separator_style.into_lua(lua),

			_ => Ok(Value::Nil),
		}
	}

	Composer::new(get)
}

fn confirm() -> Composer<ComposerGet> {
	fn get(lua: &Lua, key: &[u8]) -> mlua::Result<Value> {
		let t = &THEME.confirm;
		match key {
			b"border" => t.border.into_lua(lua),
			b"border_set" => t.border_set.load_full().into_lua(lua),
			b"title" => t.title.into_lua(lua),
			b"body" => t.body.into_lua(lua),
			b"list" => t.list.into_lua(lua),

			b"btn_yes" => t.btn_yes.into_lua(lua),
			b"btn_no" => t.btn_no.into_lua(lua),
			b"btn_labels" => {
				lua.create_sequence_from(t.btn_labels.load().iter().map(String::as_str))?.into_lua(lua)
			}

			_ => Ok(Value::Nil),
		}
	}

	Composer::new(get)
}

fn spot() -> Composer<ComposerGet> {
	fn get(lua: &Lua, key: &[u8]) -> mlua::Result<Value> {
		let t = &THEME.spot;
		match key {
			b"border" => t.border.into_lua(lua),
			b"border_set" => t.border_set.load_full().into_lua(lua),
			b"title" => t.title.into_lua(lua),

			b"tbl_col" => t.tbl_col.into_lua(lua),
			b"tbl_cell" => t.tbl_cell.into_lua(lua),

			_ => Ok(Value::Nil),
		}
	}

	Composer::new(get)
}

fn notify() -> Composer<ComposerGet> {
	fn get(lua: &Lua, key: &[u8]) -> mlua::Result<Value> {
		let t = &THEME.notify;
		match key {
			b"border_set" => t.border_set.load_full().into_lua(lua),

			b"title_info" => t.title_info.into_lua(lua),
			b"title_warn" => t.title_warn.into_lua(lua),
			b"title_error" => t.title_error.into_lua(lua),

			b"icon_info" => lua.create_string(&**t.icon_info.load())?.into_lua(lua),
			b"icon_warn" => lua.create_string(&**t.icon_warn.load())?.into_lua(lua),
			b"icon_error" => lua.create_string(&**t.icon_error.load())?.into_lua(lua),

			_ => Ok(Value::Nil),
		}
	}

	Composer::new(get)
}

fn pick() -> Composer<ComposerGet> {
	fn get(lua: &Lua, key: &[u8]) -> mlua::Result<Value> {
		let t = &THEME.pick;
		match key {
			b"border" => t.border.into_lua(lua),
			b"border_set" => t.border_set.load_full().into_lua(lua),
			b"active" => t.active.into_lua(lua),
			b"inactive" => t.inactive.into_lua(lua),

			_ => Ok(Value::Nil),
		}
	}

	Composer::new(get)
}

fn input() -> Composer<ComposerGet> {
	fn get(lua: &Lua, key: &[u8]) -> mlua::Result<Value> {
		let t = &THEME.input;
		match key {
			b"border" => t.border.into_lua(lua),
			b"border_set" => t.border_set.load_full().into_lua(lua),
			b"title" => t.title.into_lua(lua),
			b"value" => t.value.into_lua(lua),
			b"selected" => t.selected.into_lua(lua),

			_ => Ok(Value::Nil),
		}
	}

	Composer::new(get)
}

fn cmp() -> Composer<ComposerGet> {
	fn get(lua: &Lua, key: &[u8]) -> mlua::Result<Value> {
		let t = &THEME.cmp;
		match key {
			b"border" => t.border.into_lua(lua),
			b"border_set" => t.border_set.load_full().into_lua(lua),
			b"active" => t.active.into_lua(lua),
			b"inactive" => t.inactive.into_lua(lua),

			b"icon_file" => lua.create_string(&**t.icon_file.load())?.into_lua(lua),
			b"icon_folder" => lua.create_string(&**t.icon_folder.load())?.into_lua(lua),
			b"icon_command" => lua.create_string(&**t.icon_command.load())?.into_lua(lua),

			_ => Ok(Value::Nil),
		}
	}

	Composer::new(get)
}

fn tasks() -> Composer<ComposerGet> {
	fn get(lua: &Lua, key: &[u8]) -> mlua::Result<Value> {
		let t = &THEME.tasks;
		match key {
			b"border" => t.border.into_lua(lua),
			b"border_set" => t.border_set.load_full().into_lua(lua),
			b"title" => t.title.into_lua(lua),
			b"hovered" => t.hovered.into_lua(lua),

			_ => Ok(Value::Nil),
		}
	}

	Composer::new(get)
}

fn help() -> Composer<ComposerGet> {
	fn get(lua: &Lua, key: &[u8]) -> mlua::Result<Value> {
		let t = &THEME.help;
		match key {
			b"border" => t.border.into_lua(lua),
			b"border_set" => t.border_set.load_full().into_lua(lua),
			b"chord" => t.chord.into_lua(lua),
			b"action" => t.action.into_lua(lua),
			b"hovered" => t.hovered.into_lua(lua),

			_ => Ok(Value::Nil),
		}
	}

	Composer::new(get)
}

fn custom(lua: &Lua, key: &[u8]) -> mlua::Result<Value> {
	THEME.custom.load().get(str::from_utf8(key)?).map(CustomSectionArc::from).into_lua(lua)
}
