use mlua::{IntoLua, Lua, Value};

use crate::Composer;

pub(super) struct Utils;

pub fn compose(lua: &Lua, isolate: bool) -> mlua::Result<Value> {
	Composer::make(lua, 45, move |lua, key| {
		match key {
			// App
			b"id" => Utils::id(lua)?,
			b"hide" => Utils::hide(lua)?,

			// Cache
			b"file_cache" => Utils::file_cache(lua)?,

			// Call
			b"render" => Utils::render(lua)?,
			b"emit" => Utils::emit(lua)?,
			b"app_emit" => Utils::app_emit(lua)?,
			b"mgr_emit" => Utils::mgr_emit(lua)?,
			b"manager_emit" => Utils::mgr_emit(lua)?, // TODO: remove this in the future
			b"input_emit" => Utils::input_emit(lua)?,

			// Image
			b"image_info" => Utils::image_info(lua)?,
			b"image_show" => Utils::image_show(lua)?,
			b"image_precache" => Utils::image_precache(lua)?,

			// JSON
			b"json_encode" => Utils::json_encode(lua)?,
			b"json_decode" => Utils::json_decode(lua)?,

			// Layout
			b"which" => Utils::which(lua)?,
			b"input" => Utils::input(lua)?,
			b"confirm" => Utils::confirm(lua)?,
			b"notify" => Utils::notify(lua)?,

			// Log
			b"dbg" => Utils::dbg(lua)?,
			b"err" => Utils::err(lua)?,

			// Preview
			b"preview_code" => Utils::preview_code(lua)?,
			b"preview_widget" => Utils::preview_widget(lua)?,
			b"preview_widgets" => Utils::preview_widget(lua)?, // TODO: remove this in the future

			// Process
			b"proc_info" => Utils::proc_info(lua)?,

			// Spot
			b"spot_table" => Utils::spot_table(lua)?,
			b"spot_widgets" => Utils::spot_widgets(lua)?,

			// Sync
			b"sync" => Utils::sync(lua, isolate)?,
			b"chan" => Utils::chan(lua)?,
			b"join" => Utils::join(lua)?,
			b"select" => Utils::select(lua)?,

			// Target
			b"target_os" => Utils::target_os(lua)?,
			b"target_family" => Utils::target_family(lua)?,

			// Text
			b"hash" => Utils::hash(lua)?,
			b"quote" => Utils::quote(lua)?,
			b"truncate" => Utils::truncate(lua)?,
			b"clipboard" => Utils::clipboard(lua)?,

			// Time
			b"time" => Utils::time(lua)?,
			b"sleep" => Utils::sleep(lua)?,

			// User
			#[cfg(unix)]
			b"uid" => Utils::uid(lua)?,
			#[cfg(unix)]
			b"gid" => Utils::gid(lua)?,
			#[cfg(unix)]
			b"user_name" => Utils::user_name(lua)?,
			#[cfg(unix)]
			b"group_name" => Utils::group_name(lua)?,
			#[cfg(unix)]
			b"host_name" => Utils::host_name(lua)?,

			_ => return Ok(Value::Nil),
		}
		.into_lua(lua)
	})
}
