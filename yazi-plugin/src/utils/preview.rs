use mlua::{AnyUserData, Function, IntoLuaMulti, Lua, Table, Value};
use yazi_config::{YAZI, preview::PreviewWrap};
use yazi_macro::emit;
use yazi_shared::{errors::PeekError, event::Cmd};

use super::Utils;
use crate::{elements::{Area, Rect, Renderable, Text, WRAP, WRAP_NO}, external::Highlighter, file::FileRef};

#[derive(Debug, Default)]
pub struct PreviewLock {
	pub url:  yazi_shared::url::Url,
	pub cha:  yazi_fs::cha::Cha,
	pub mime: String,

	pub skip: usize,
	pub area: Rect,
	pub data: Vec<Renderable>,
}

impl TryFrom<Table> for PreviewLock {
	type Error = mlua::Error;

	fn try_from(t: Table) -> Result<Self, Self::Error> {
		let file: FileRef = t.raw_get("file")?;
		Ok(Self {
			url:  file.url_owned(),
			cha:  file.cha,
			mime: t.raw_get("mime")?,

			skip: t.raw_get("skip")?,
			area: t.raw_get("area")?,
			data: Default::default(),
		})
	}
}

impl Utils {
	pub(super) fn preview_code(lua: &Lua) -> mlua::Result<Function> {
		lua.create_async_function(|lua, t: Table| async move {
			let area: Area = t.raw_get("area")?;
			let mut lock = PreviewLock::try_from(t)?;

			let inner = match Highlighter::new(&lock.url).highlight(lock.skip, area.size()).await {
				Ok(text) => text,
				Err(e @ PeekError::Exceed(max)) => return (e.to_string(), max).into_lua_multi(&lua),
				Err(e @ PeekError::Unexpected(_)) => {
					return (e.to_string(), Value::Nil).into_lua_multi(&lua);
				}
			};

			lock.data = vec![Renderable::Text(Text {
				area,
				inner,
				wrap: if YAZI.preview.wrap == PreviewWrap::Yes { WRAP } else { WRAP_NO },
				scroll: Default::default(),
			})];

			emit!(Call(Cmd::new("mgr:update_peeked").with_any("lock", lock)));
			(Value::Nil, Value::Nil).into_lua_multi(&lua)
		})
	}

	pub(super) fn preview_widgets(lua: &Lua) -> mlua::Result<Function> {
		lua.create_async_function(|_, (t, widgets): (Table, Vec<AnyUserData>)| async move {
			let mut lock = PreviewLock::try_from(t)?;
			lock.data = widgets.into_iter().map(Renderable::try_from).collect::<mlua::Result<_>>()?;

			emit!(Call(Cmd::new("mgr:update_peeked").with_any("lock", lock)));
			Ok(())
		})
	}
}
