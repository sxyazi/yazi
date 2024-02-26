use mlua::{AnyUserData, ExternalError, FromLua, IntoLua, Lua, Table, UserData, UserDataMethods, Value};

use super::{Span, Style};

const LEFT: u8 = 0;
const CENTER: u8 = 1;
const RIGHT: u8 = 2;

#[derive(Clone, FromLua)]
pub struct Line(pub(super) ratatui::text::Line<'static>);

impl<'a> TryFrom<Table<'a>> for Line {
	type Error = mlua::Error;

	fn try_from(tb: Table) -> Result<Self, Self::Error> {
		let seq: Vec<_> = tb.sequence_values().filter_map(|v| v.ok()).collect();
		let mut spans = Vec::with_capacity(seq.len());
		for value in seq {
			if let Value::UserData(ud) = value {
				if let Ok(span) = ud.take::<Span>() {
					spans.push(span.0);
				} else if let Ok(line) = ud.take::<Line>() {
					spans.extend(line.0.spans.into_iter().collect::<Vec<_>>());
				} else {
					return Err("expected a table of Spans or Lines".into_lua_err());
				}
			}
		}
		Ok(Self(ratatui::text::Line::from(spans)))
	}
}

impl Line {
	pub fn install(lua: &Lua, ui: &Table) -> mlua::Result<()> {
		let new = lua.create_function(|_, (_, value): (Table, Value)| {
			if let Value::Table(tb) = value {
				return Line::try_from(tb);
			}
			Err("expected a table of Spans or Lines".into_lua_err())
		})?;

		let line = lua.create_table_from([
			// Alignment
			("LEFT", LEFT.into_lua(lua)?),
			("CENTER", CENTER.into_lua(lua)?),
			("RIGHT", RIGHT.into_lua(lua)?),
		])?;

		line.set_metatable(Some(lua.create_table_from([("__call", new)])?));

		ui.raw_set("Line", line)
	}
}

impl UserData for Line {
	fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
		methods.add_function("width", |_, ud: AnyUserData| Ok(ud.borrow_mut::<Self>()?.0.width()));
		methods.add_function("style", |_, (ud, value): (AnyUserData, Value)| {
			{
				let mut me = ud.borrow_mut::<Self>()?;
				me.0.style = match value {
					Value::Nil => me.0.style.patch(ratatui::style::Style::reset()),
					Value::Table(tb) => me.0.style.patch(Style::from(tb).0),
					Value::UserData(ud) => me.0.style.patch(ud.borrow::<Style>()?.0),
					_ => return Err("expected a Style or Table or nil".into_lua_err()),
				};
			}
			Ok(ud)
		});
		methods.add_function("align", |_, (ud, align): (AnyUserData, u8)| {
			ud.borrow_mut::<Self>()?.0.alignment = Some(match align {
				CENTER => ratatui::layout::Alignment::Center,
				RIGHT => ratatui::layout::Alignment::Right,
				_ => ratatui::layout::Alignment::Left,
			});
			Ok(ud)
		});
	}
}
