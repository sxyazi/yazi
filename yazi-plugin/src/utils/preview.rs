use mlua::{AnyUserData, IntoLuaMulti, Lua, Table, Value};
use yazi_shared::{emit, event::Exec, Layer, PeekError};

use super::Utils;
use crate::{bindings::{FileRef, Window}, cast_to_renderable, elements::{Paragraph, RectRef, Renderable}, external::{self, Highlighter}};

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
		let file: FileRef = t.get("file")?;
		Ok(Self {
			url:    file.url(),
			cha:    file.cha,
			skip:   t.get("skip")?,
			window: t.get("window")?,
			data:   Default::default(),
		})
	}
}

impl Utils {
	pub(super) fn preview(lua: &Lua, ya: &Table) -> mlua::Result<()> {
		ya.set(
			"preview_code",
			lua.create_async_function(|lua, t: Table| async move {
				let area: RectRef = t.get("area")?;
				let mut lock = PreviewLock::try_from(t)?;

				let text =
					match Highlighter::new(&lock.url).highlight(lock.skip, area.height as usize).await {
						Ok(text) => text,
						Err(PeekError::Exceed(max)) => return (false, max).into_lua_multi(lua),
						Err(_) => return (false, Value::Nil).into_lua_multi(lua),
					};
				lock.data = vec![Box::new(Paragraph { area: *area, text, ..Default::default() })];

				emit!(Call(Exec::call("preview", vec![]).with_data(lock).vec(), Layer::Manager));
				(true, Value::Nil).into_lua_multi(lua)
			})?,
		)?;

		ya.set(
			"preview_archive",
			lua.create_async_function(|lua, t: Table| async move {
				let area: RectRef = t.get("area")?;
				let mut lock = PreviewLock::try_from(t)?;

				let lines: Vec<_> = match external::lsar(&lock.url, lock.skip, area.height as usize).await {
					Ok(items) => items.into_iter().map(|f| ratatui::text::Line::from(f.name)).collect(),
					Err(PeekError::Exceed(max)) => return (false, max).into_lua_multi(lua),
					Err(_) => return (false, Value::Nil).into_lua_multi(lua),
				};

				lock.data = vec![Box::new(Paragraph {
					area: *area,
					text: ratatui::text::Text::from(lines),
					..Default::default()
				})];

				emit!(Call(Exec::call("preview", vec![]).with_data(lock).vec(), Layer::Manager));
				(true, Value::Nil).into_lua_multi(lua)
			})?,
		)?;

		ya.set(
			"preview_widgets",
			lua.create_async_function(|_, (t, widgets): (Table, Vec<AnyUserData>)| async move {
				let mut lock = PreviewLock::try_from(t)?;
				lock.data = widgets.into_iter().filter_map(cast_to_renderable).collect();

				emit!(Call(Exec::call("preview", vec![]).with_data(lock).vec(), Layer::Manager));
				Ok(())
			})?,
		)?;

		Ok(())
	}
}
