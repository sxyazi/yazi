use mlua::{ExternalError, Function, IntoLuaMulti, Lua, Table, Value};
use yazi_binding::{Error, elements::{Area, Renderable, Text}};
use yazi_core::{Highlighter, MgrProxy, tab::PreviewLock};
use yazi_fs::FsUrl;
use yazi_runner::previewer::PeekError;
use yazi_shared::url::AsUrl;

use super::Utils;

impl Utils {
	// TODO:
	// return (Text?, PeekError?) instead of (String, usize?) to align with other
	// APIs, and allow users to use the Text for more flexible preview
	pub(super) fn preview_code(lua: &Lua) -> mlua::Result<Function> {
		lua.create_async_function(|lua, t: Table| async move {
			let area: Area = t.raw_get("area")?;
			let mut lock = PreviewLock::try_from(t)?;

			let path = lock.url.as_url().unified_path();
			let inner = match Highlighter::oneshot(path, lock.skip, area.size()).await {
				Ok(text) => text,
				Err(e @ PeekError::Exceeded(max)) => return (e, max).into_lua_multi(&lua),
				Err(e) => {
					return e.into_lua_multi(&lua);
				}
			};

			lock.data = vec![Renderable::Text(Text { area, inner, ..Default::default() })];

			MgrProxy::update_peeked(lock);
			().into_lua_multi(&lua)
		})
	}

	pub(super) fn preview_widget(lua: &Lua) -> mlua::Result<Function> {
		lua.create_async_function(|_, (t, value): (Table, Value)| async move {
			let mut lock = PreviewLock::try_from(t)?;
			lock.data = match value {
				Value::Nil => vec![],
				Value::Table(tbl) => tbl.sequence_values::<Renderable>().collect::<mlua::Result<_>>()?,
				Value::UserData(ud) => match Renderable::try_from(&ud) {
					Ok(r) => vec![r],
					Err(e) => {
						if let Ok(err) = ud.take::<Error>() {
							vec![
								Renderable::Clear(yazi_binding::elements::Clear { area: lock.area.into() }),
								Renderable::from(err).with_area(lock.area),
							]
						} else {
							Err(e)?
						}
					}
				},
				_ => Err("preview widget must be a renderable element or a table of them".into_lua_err())?,
			};

			MgrProxy::update_peeked(lock);
			Ok(())
		})
	}
}
