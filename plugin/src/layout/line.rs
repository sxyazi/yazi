use mlua::{AnyUserData, FromLua, Lua, Table, UserData, UserDataMethods, Value};

use super::{Span, Style};
use crate::{GLOBALS, LUA};

#[derive(Clone)]
pub(crate) struct Line(pub(super) ratatui::text::Line<'static>);

impl Line {
	pub(crate) fn install() -> mlua::Result<()> {
		let ui: Table = GLOBALS.get("ui")?;
		ui.set(
			"Line",
			LUA.create_function(|_, value: Value| {
				if let Value::Table(tbl) = value {
					let seq: Vec<_> = tbl.sequence_values().filter_map(|v| v.ok()).collect();
					let mut spans = Vec::with_capacity(seq.len());
					for value in seq {
						if let Value::UserData(ud) = value {
							if let Ok(span) = ud.take::<Span>() {
								spans.push(span.0);
							} else if let Ok(line) = ud.take::<Line>() {
								spans.extend(line.0.spans.into_iter().collect::<Vec<_>>());
							} else {
								return Err(mlua::Error::external("expected a table of Spans or Lines"));
							}
						}
					}
					return Ok(Self(ratatui::text::Line::from(spans)));
				}

				Err(mlua::Error::external("expected a table of Spans or Lines"))
			})?,
		)
	}
}

impl<'lua> FromLua<'lua> for Line {
	fn from_lua(value: Value<'lua>, _: &'lua Lua) -> mlua::Result<Self> {
		match value {
			Value::UserData(ud) => Ok(ud.borrow::<Self>()?.clone()),
			_ => Err(mlua::Error::FromLuaConversionError {
				from:    value.type_name(),
				to:      "Line",
				message: Some("expected a Line".to_string()),
			}),
		}
	}
}

impl UserData for Line {
	fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
		methods.add_function("width", |_, ud: AnyUserData| Ok(ud.borrow_mut::<Self>()?.0.width()));
		methods.add_function("style", |_, (ud, value): (AnyUserData, Value)| {
			{
				let mut me = ud.borrow_mut::<Self>()?;
				match value {
					Value::Nil => me.0.reset_style(),
					Value::Table(tbl) => me.0.patch_style(Style::from(tbl).0),
					Value::UserData(ud) => me.0.patch_style(ud.borrow::<Style>()?.0),
					_ => return Err(mlua::Error::external("expected a Style or Table or nil")),
				}
			}
			Ok(ud)
		});
		methods.add_function("align", |_, (ud, align): (AnyUserData, u8)| {
			ud.borrow_mut::<Self>()?.0.alignment = Some(match align {
				1 => ratatui::prelude::Alignment::Center,
				2 => ratatui::prelude::Alignment::Right,
				_ => ratatui::prelude::Alignment::Left,
			});
			Ok(ud)
		});
	}
}
