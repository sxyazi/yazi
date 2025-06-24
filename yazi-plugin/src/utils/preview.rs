use mlua::{AnyUserData, ExternalError, Function, IntoLuaMulti, Lua, Table, Value};
use yazi_binding::Error;
use yazi_config::YAZI;
use yazi_macro::emit;
use yazi_shared::{errors::PeekError, event::Cmd};

use super::Utils;
use crate::{elements::{Area, Rect, Renderable, Text}, external::Highlighter, file::FileRef};

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
					return e.to_string().into_lua_multi(&lua);
				}
			};

			lock.data = vec![Renderable::Text(Text {
				area,
				inner,
				wrap: YAZI.preview.wrap.into(),
				scroll: Default::default(),
			})];

			emit!(Call(Cmd::new("mgr:update_peeked").with_any("lock", lock)));
			().into_lua_multi(&lua)
		})
	}

	pub(super) fn preview_widget(lua: &Lua) -> mlua::Result<Function> {
		lua.create_async_function(|_, (t, value): (Table, Value)| async move {
			let mut lock = PreviewLock::try_from(t)?;
			lock.data = match value {
				Value::Nil => vec![],
				Value::Table(tbl) => tbl
					.sequence_values::<AnyUserData>()
					.map(|ud| ud.and_then(Renderable::try_from))
					.collect::<mlua::Result<_>>()?,
				Value::UserData(ud) => match Renderable::try_from(&ud) {
					Ok(r) => vec![r],
					Err(e) => {
						if let Ok(err) = ud.take::<Error>() {
							vec![(lock.area, err).into()]
						} else {
							Err(e)?
						}
					}
				},
				_ => Err("preview widget must be a renderable element or a table of them".into_lua_err())?,
			};

			emit!(Call(Cmd::new("mgr:update_peeked").with_any("lock", lock)));
			Ok(())
		})
	}
}
