use mlua::{AnyUserData, IntoLuaMulti, Lua, Table, Value};
use yazi_config::{PREVIEW, preview::PreviewWrap};
use yazi_macro::emit;
use yazi_shared::{Layer, errors::PeekError, event::Cmd};

use super::Utils;
use crate::{bindings::Window, cast_to_renderable, elements::{Rect, Renderable, Text, WRAP, WRAP_NO}, external::Highlighter, file::FileRef};

pub struct PreviewLock {
	pub url:  yazi_shared::fs::Url,
	pub cha:  yazi_shared::fs::Cha,
	pub mime: String,

	pub skip:   usize,
	pub window: Window,
	pub data:   Vec<Box<dyn Renderable + Send>>,
}

impl TryFrom<Table> for PreviewLock {
	type Error = mlua::Error;

	fn try_from(t: Table) -> Result<Self, Self::Error> {
		let file: FileRef = t.raw_get("file")?; // TODO: use `_file` instead of `file`
		Ok(Self {
			url:  file.url_owned(),
			cha:  file.cha,
			mime: t.raw_get("_mime")?,

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
				let area: Rect = t.raw_get("area")?;
				let mut lock = PreviewLock::try_from(t)?;

				let inner = match Highlighter::new(&lock.url).highlight(lock.skip, *area).await {
					Ok(text) => text,
					Err(e @ PeekError::Exceed(max)) => return (e.to_string(), max).into_lua_multi(&lua),
					Err(e @ PeekError::Unexpected(_)) => {
						return (e.to_string(), Value::Nil).into_lua_multi(&lua);
					}
				};

				lock.data = vec![Box::new(Text {
					area,
					inner,
					wrap: if PREVIEW.wrap == PreviewWrap::Yes { WRAP } else { WRAP_NO },
				})];

				emit!(Call(Cmd::new("update_peeked").with_any("lock", lock), Layer::Manager));
				(Value::Nil, Value::Nil).into_lua_multi(&lua)
			})?,
		)?;

		ya.raw_set(
			"preview_widgets",
			lua.create_async_function(|_, (t, widgets): (Table, Vec<AnyUserData>)| async move {
				let mut lock = PreviewLock::try_from(t)?;
				lock.data = widgets.into_iter().filter_map(|ud| cast_to_renderable(&ud)).collect();

				emit!(Call(Cmd::new("update_peeked").with_any("lock", lock), Layer::Manager));
				Ok(())
			})?,
		)?;

		Ok(())
	}
}
