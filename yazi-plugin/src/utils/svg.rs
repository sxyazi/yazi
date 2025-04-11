use std::sync::Arc;

use mlua::{Function, IntoLua, Lua, Value};
use resvg::usvg::fontdb::Database;
use yazi_adapter::GLOBAL_OPTIONS;
use yazi_binding::UrlRef;

use super::Utils;
use crate::bindings::SvgInfo;

impl Utils {
	pub(super) fn svg_info(lua: &Lua) -> mlua::Result<Function> {
		lua.create_async_function(|lua, url: UrlRef| async move {
			if let Ok(info) = yazi_adapter::SvgInfo::new(&url).await {
				SvgInfo::from(info).into_lua(&lua)
			} else {
				Value::Nil.into_lua(&lua)
			}
		})
	}

	pub(super) fn set_svg_font_family(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|_, (family, font_name): (mlua::String, mlua::String)| {
			let font = font_name.to_str()?;

			Self::with_mut_fontdb(|db| {
				match family.to_str()?.as_ref() {
					"serif" => db.set_serif_family(&*font),
					"sans-serif" => db.set_sans_serif_family(&*font),
					"cursive" => db.set_cursive_family(&*font),
					"fantasy" => db.set_fantasy_family(&*font),
					"monospace" => db.set_monospace_family(&*font),
					_ => return Err(mlua::Error::external("Invalid font family")),
				}
				Ok(())
			})
		})
	}

	pub(super) fn load_system_fonts(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|_, ()| {
			Self::with_mut_fontdb(|db| {
				db.load_system_fonts();
				Ok(())
			})
		})
	}

	pub(super) fn load_font_file(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|_, path: mlua::String| {
			Self::with_mut_fontdb(|db| {
				db.load_font_file(&*path.to_str()?)
					.map_err(|e| mlua::Error::external(format!("Failed to load font file: {e}")))
			})
		})
	}

	pub(super) fn load_fonts_dir(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|_, dir: mlua::String| {
			Self::with_mut_fontdb(|db| {
				db.load_fonts_dir(&*dir.to_str()?);
				Ok(())
			})
		})
	}

	fn with_mut_fontdb<F>(f: F) -> mlua::Result<()>
	where
		F: FnOnce(&mut Database) -> mlua::Result<()>,
	{
		let mut options = GLOBAL_OPTIONS
			.write()
			.map_err(|e| mlua::Error::external(format!("RwLock poisoned: {}", e)))?;
		f(Arc::make_mut(&mut options.fontdb))
	}
}
