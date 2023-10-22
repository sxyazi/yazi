use yazi_config::theme::Color;
use mlua::{AnyUserData, FromLua, Lua, Table, UserData, UserDataMethods, Value};

use super::Style;
use crate::{GLOBALS, LUA};

#[derive(Clone)]
pub(crate) struct Span(pub(super) ratatui::text::Span<'static>);

impl Span {
	pub(crate) fn install() -> mlua::Result<()> {
		let ui: Table = GLOBALS.get("ui")?;
		ui.set(
			"Span",
			LUA.create_function(|_, content: String| Ok(Self(ratatui::text::Span::raw(content))))?,
		)
	}
}

impl<'lua> FromLua<'lua> for Span {
	fn from_lua(value: Value<'lua>, _: &'lua Lua) -> mlua::Result<Self> {
		match value {
			Value::UserData(ud) => Ok(ud.borrow::<Self>()?.clone()),
			_ => Err(mlua::Error::FromLuaConversionError {
				from:    value.type_name(),
				to:      "Span",
				message: Some("expected a Span".to_string()),
			}),
		}
	}
}

impl UserData for Span {
	fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
		methods.add_function("fg", |_, (ud, color): (AnyUserData, String)| {
			ud.borrow_mut::<Self>()?.0.style.fg = Color::try_from(color).ok().map(Into::into);
			Ok(ud)
		});
		methods.add_function("bg", |_, (ud, color): (AnyUserData, String)| {
			ud.borrow_mut::<Self>()?.0.style.bg = Color::try_from(color).ok().map(Into::into);
			Ok(ud)
		});
		methods.add_function("bold", |_, ud: AnyUserData| {
			ud.borrow_mut::<Self>()?.0.style.add_modifier |= ratatui::style::Modifier::BOLD;
			Ok(ud)
		});
		methods.add_function("dim", |_, ud: AnyUserData| {
			ud.borrow_mut::<Self>()?.0.style.add_modifier |= ratatui::style::Modifier::DIM;
			Ok(ud)
		});
		methods.add_function("italic", |_, ud: AnyUserData| {
			ud.borrow_mut::<Self>()?.0.style.add_modifier |= ratatui::style::Modifier::ITALIC;
			Ok(ud)
		});
		methods.add_function("underline", |_, ud: AnyUserData| {
			ud.borrow_mut::<Self>()?.0.style.add_modifier |= ratatui::style::Modifier::UNDERLINED;
			Ok(ud)
		});
		methods.add_function("blink", |_, ud: AnyUserData| {
			ud.borrow_mut::<Self>()?.0.style.add_modifier |= ratatui::style::Modifier::SLOW_BLINK;
			Ok(ud)
		});
		methods.add_function("blink_rapid", |_, ud: AnyUserData| {
			ud.borrow_mut::<Self>()?.0.style.add_modifier |= ratatui::style::Modifier::RAPID_BLINK;
			Ok(ud)
		});
		methods.add_function("hidden", |_, ud: AnyUserData| {
			ud.borrow_mut::<Self>()?.0.style.add_modifier |= ratatui::style::Modifier::HIDDEN;
			Ok(ud)
		});
		methods.add_function("crossed", |_, ud: AnyUserData| {
			ud.borrow_mut::<Self>()?.0.style.add_modifier |= ratatui::style::Modifier::CROSSED_OUT;
			Ok(ud)
		});
		methods.add_function("reset", |_, ud: AnyUserData| {
			ud.borrow_mut::<Self>()?.0.style.add_modifier = ratatui::style::Modifier::empty();
			Ok(ud)
		});

		methods.add_function("style", |_, (ud, value): (AnyUserData, Value)| {
			ud.borrow_mut::<Self>()?.0.style = match value {
				Value::Nil => ratatui::style::Style::default(),
				Value::Table(tbl) => Style::from(tbl).0,
				Value::UserData(ud) => ud.borrow::<Style>()?.0,
				_ => return Err(mlua::Error::external("expected a Style or Table or nil")),
			};
			Ok(ud)
		});
	}
}
