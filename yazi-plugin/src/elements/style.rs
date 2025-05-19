use std::str::FromStr;

use mlua::{AnyUserData, ExternalError, ExternalResult, IntoLua, Lua, MetaMethod, Table, UserData, UserDataMethods, Value};
use yazi_shared::theme::Color;

#[derive(Clone, Copy, Default)]
pub struct Style(pub(super) ratatui::style::Style);

impl Style {
	pub fn compose(lua: &Lua) -> mlua::Result<Value> {
		let new = lua.create_function(|_, (_, value): (Table, Value)| Self::try_from(value))?;

		let style = lua.create_table()?;
		style.set_metatable(Some(lua.create_table_from([(MetaMethod::Call.name(), new)])?));

		style.into_lua(lua)
	}
}

impl TryFrom<Table> for Style {
	type Error = mlua::Error;

	fn try_from(value: Table) -> Result<Self, Self::Error> {
		let mut style = ratatui::style::Style::default();
		if let Ok(fg) = value.raw_get::<mlua::String>("fg") {
			style.fg = Some(Color::from_str(&fg.to_str()?).into_lua_err()?.into());
		}
		if let Ok(bg) = value.raw_get::<mlua::String>("bg") {
			style.bg = Some(Color::from_str(&bg.to_str()?).into_lua_err()?.into());
		}
		style.add_modifier =
			ratatui::style::Modifier::from_bits_truncate(value.raw_get("modifier").unwrap_or_default());
		Ok(Self(style))
	}
}

impl TryFrom<Value> for Style {
	type Error = mlua::Error;

	fn try_from(value: Value) -> Result<Self, Self::Error> {
		Ok(Self(match value {
			Value::Nil => Default::default(),
			Value::Table(tb) => Self::try_from(tb)?.0,
			Value::UserData(ud) => ud.borrow::<Self>()?.0,
			_ => Err("expected a Style or Table or nil".into_lua_err())?,
		}))
	}
}

impl From<yazi_shared::theme::Style> for Style {
	fn from(value: yazi_shared::theme::Style) -> Self { Self(value.into()) }
}

impl UserData for Style {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		crate::impl_style_shorthands!(methods, 0);

		methods.add_function_mut("patch", |_, (ud, value): (AnyUserData, Value)| {
			{
				let mut me = ud.borrow_mut::<Self>()?;
				me.0 = me.0.patch(Self::try_from(value)?.0);
			}
			Ok(ud)
		})
	}
}
