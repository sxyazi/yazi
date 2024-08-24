use std::str::FromStr;

use mlua::{AnyUserData, ExternalError, ExternalResult, Lua, Table, UserData, UserDataMethods, Value};
use yazi_shared::theme::Color;

#[derive(Clone, Copy, Default)]
pub struct Style(pub(super) ratatui::style::Style);

impl Style {
	pub fn install(lua: &Lua, ui: &Table) -> mlua::Result<()> {
		let new = lua.create_function(|_, ()| Ok(Self::default()))?;

		let style = lua.create_table()?;
		style.set_metatable(Some(lua.create_table_from([("__call", new)])?));

		ui.raw_set("Style", style)
	}
}

impl From<yazi_shared::theme::Style> for Style {
	fn from(value: yazi_shared::theme::Style) -> Self { Self(value.into()) }
}

impl<'a> TryFrom<Table<'a>> for Style {
	type Error = mlua::Error;

	fn try_from(value: Table<'a>) -> Result<Self, Self::Error> {
		let mut style = ratatui::style::Style::default();
		if let Ok(fg) = value.raw_get::<_, mlua::String>("fg") {
			style.fg = Some(Color::from_str(fg.to_str()?).into_lua_err()?.into());
		}
		if let Ok(bg) = value.raw_get::<_, mlua::String>("bg") {
			style.bg = Some(Color::from_str(bg.to_str()?).into_lua_err()?.into());
		}
		style.add_modifier =
			ratatui::style::Modifier::from_bits_truncate(value.raw_get("modifier").unwrap_or_default());
		Ok(Self(style))
	}
}

impl UserData for Style {
	fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
		crate::impl_style_shorthands!(methods, 0);

		methods.add_function("patch", |_, (ud, value): (AnyUserData, Value)| {
			{
				let mut me = ud.borrow_mut::<Self>()?;
				me.0 = me.0.patch(match value {
					Value::Table(tb) => Style::try_from(tb)?.0,
					Value::UserData(ud) => ud.borrow::<Style>()?.0,
					_ => return Err("expected a Style or Table".into_lua_err()),
				});
			}
			Ok(ud)
		})
	}
}
