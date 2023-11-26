use ansi_to_tui::IntoText;
use mlua::{AnyUserData, ExternalResult, IntoLuaMulti, Lua, Table, Value};
use yazi_config::PREVIEW;
use yazi_shared::{emit, event::Exec, Layer, PeekError};

use super::Utils;
use crate::{bindings::FileRef, cast_to_renderable, elements::{Paragraph, RectRef, Renderable}, external::{self, Highlighter}};

pub struct PreviewLock {
	pub url: yazi_shared::fs::Url,
	pub cha: yazi_shared::fs::Cha,

	pub skip: usize,
	pub data: Vec<Box<dyn Renderable + Send>>,
}

impl Utils {
	pub(super) fn preview(lua: &Lua, ya: &Table) -> mlua::Result<()> {
		ya.set(
			"preview_code",
			lua.create_async_function(
				|lua, (area, file, skip): (RectRef, FileRef, usize)| async move {
					let s = match Highlighter::new(&file.url).highlight(skip, area.height as usize).await {
						Ok(s) => s.replace('\t', &" ".repeat(PREVIEW.tab_size as usize)),
						Err(PeekError::Exceed(max)) => return (false, max).into_lua_multi(lua),
						Err(_) => return (false, Value::Nil).into_lua_multi(lua),
					};

					let lock = PreviewLock {
						url: file.url(),
						cha: file.cha,
						skip,
						data: vec![Box::new(Paragraph {
							area: *area,
							text: s.into_text().into_lua_err()?,
							..Default::default()
						})],
					};

					emit!(Call(Exec::call("preview", vec![]).with_data(lock).vec(), Layer::Manager));
					(true, Value::Nil).into_lua_multi(lua)
				},
			)?,
		)?;

		// TODO: remove this once "archives as directories" feature is implemented
		ya.set(
			"preview_archive",
			lua.create_async_function(
				|lua, (area, file, skip): (RectRef, FileRef, usize)| async move {
					let lines: Vec<_> = match external::lsar(&file.url, skip, area.height as usize).await {
						Ok(items) => items.into_iter().map(|f| ratatui::text::Line::from(f.name)).collect(),
						Err(PeekError::Exceed(max)) => return (false, max).into_lua_multi(lua),
						Err(_) => return (false, Value::Nil).into_lua_multi(lua),
					};

					let lock = PreviewLock {
						url: file.url(),
						cha: file.cha,
						skip,
						data: vec![Box::new(Paragraph {
							area: *area,
							text: ratatui::text::Text::from(lines),
							..Default::default()
						})],
					};

					emit!(Call(Exec::call("preview", vec![]).with_data(lock).vec(), Layer::Manager));
					(true, Value::Nil).into_lua_multi(lua)
				},
			)?,
		)?;

		ya.set(
			"preview_widgets",
			lua.create_async_function(
				|_, (file, skip, widgets): (FileRef, usize, Vec<AnyUserData>)| async move {
					let lock = PreviewLock {
						url: file.url(),
						cha: file.cha,
						skip,
						data: widgets.into_iter().filter_map(cast_to_renderable).collect(),
					};
					emit!(Call(Exec::call("preview", vec![]).with_data(lock).vec(), Layer::Manager));
					Ok(())
				},
			)?,
		)?;

		Ok(())
	}
}
