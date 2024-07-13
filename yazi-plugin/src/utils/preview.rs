use mlua::{AnyUserData, IntoLuaMulti, Lua, Table, Value};
use yazi_shared::{emit, event::Cmd, Layer, PeekError};

use super::Utils;
use crate::{bindings::Window, cast_to_renderable, elements::{Paragraph, RectRef, Renderable}, external::Highlighter, file::FileRef};

pub struct PreviewLock {
	pub url: yazi_shared::fs::Url,
	pub cha: yazi_shared::fs::Cha,

	pub skip:   usize,
	pub window: Window,
	pub data:   Vec<Box<dyn Renderable + Send>>,
}

impl<'a> TryFrom<Table<'a>> for PreviewLock {
	type Error = mlua::Error;

	fn try_from(t: Table) -> Result<Self, Self::Error> {
		let file: FileRef = t.raw_get("file")?;
		Ok(Self {
			url:    file.url(),
			cha:    file.cha,
			skip:   t.raw_get("skip")?,
			window: t.raw_get("window")?,
			data:   Default::default(),
		})
	}
}

impl Utils {
	pub(super) fn preview(lua: &Lua, ya: &Table) -> mlua::Result<()> {
		ya.raw_set(
			"preview_code",
			lua.create_async_function(|lua, t: Table| async move {
				let area: RectRef = t.raw_get("area")?;
				let mut lock = PreviewLock::try_from(t)?;

				let text =
					match Highlighter::new(&lock.url).highlight(lock.skip, area.height as usize).await {
						Ok(text) => text,
						Err(PeekError::Exceed(max)) => return (false, max).into_lua_multi(lua),
						Err(_) => return (false, Value::Nil).into_lua_multi(lua),
					};
				lock.data = vec![Box::new(Paragraph { area: *area, text, ..Default::default() })];

				emit!(Call(Cmd::new("preview").with_any("lock", lock), Layer::Manager));
				(true, Value::Nil).into_lua_multi(lua)
			})?,
		)?;

		ya.raw_set(
			"preview_widgets",
			lua.create_async_function(|_, (t, widgets): (Table, Vec<AnyUserData>)| async move {
				let mut lock = PreviewLock::try_from(t)?;
				lock.data = widgets.into_iter().filter_map(|ud| cast_to_renderable(&ud)).collect();

				emit!(Call(Cmd::new("preview").with_any("lock", lock), Layer::Manager));
				Ok(())
			})?,
		)?;

		Ok(())
	}
}
