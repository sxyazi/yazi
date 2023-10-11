use config::theme::Color;
use mlua::{AnyUserData, FromLua, Lua, Table, UserData, UserDataMethods, Value};
use tracing::info;

use crate::{GLOBALS, LUA};

#[derive(Clone, Copy, Default)]
pub(crate) struct Style(pub(super) ratatui::style::Style);

impl Style {
	pub(crate) fn install() -> mlua::Result<()> {
		let ui: Table = GLOBALS.get("ui")?;
		ui.set("Style", LUA.create_function(|_, ()| Ok(Self::default()))?)
	}
}

impl<'a> From<Table<'a>> for Style {
	fn from(value: Table) -> Self {
		let mut style = ratatui::style::Style::default();
		if let Ok(fg) = value.get::<_, String>("fg") {
			style.fg = Color::try_from(fg).ok().map(Into::into);
		}
		if let Ok(bg) = value.get::<_, String>("bg") {
			style.bg = Color::try_from(bg).ok().map(Into::into);
		}
		style.add_modifier = ratatui::style::Modifier::from_bits_truncate(
			value.get::<_, u16>("modifier").unwrap_or_default(),
		);
		Self(style)
	}
}

impl<'lua> FromLua<'lua> for Style {
	fn from_lua(value: Value<'lua>, _: &'lua Lua) -> mlua::Result<Self> {
		match value {
			Value::UserData(ud) => Ok(*ud.borrow::<Self>()?),
			_ => Err(mlua::Error::FromLuaConversionError {
				from:    value.type_name(),
				to:      "Style",
				message: Some("expected a Style".to_string()),
			}),
		}
	}
}

impl UserData for Style {
	fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
		methods.add_function("fg", |_, (ud, color): (AnyUserData, String)| {
			ud.borrow_mut::<Self>()?.0.fg = Color::try_from(color).ok().map(Into::into);
			info!("fg: {:?}", ud.borrow::<Self>()?.0.fg);
			Ok(ud)
		});
		methods.add_function("bg", |_, (ud, color): (AnyUserData, String)| {
			ud.borrow_mut::<Self>()?.0.bg = Color::try_from(color).ok().map(Into::into);
			Ok(ud)
		});
		methods.add_function("bold", |_, ud: AnyUserData| {
			ud.borrow_mut::<Self>()?.0.add_modifier |= ratatui::style::Modifier::BOLD;
			Ok(ud)
		});
		methods.add_function("dim", |_, ud: AnyUserData| {
			ud.borrow_mut::<Self>()?.0.add_modifier |= ratatui::style::Modifier::DIM;
			Ok(ud)
		});
		methods.add_function("italic", |_, ud: AnyUserData| {
			ud.borrow_mut::<Self>()?.0.add_modifier |= ratatui::style::Modifier::ITALIC;
			Ok(ud)
		});
		methods.add_function("underline", |_, ud: AnyUserData| {
			ud.borrow_mut::<Self>()?.0.add_modifier |= ratatui::style::Modifier::UNDERLINED;
			Ok(ud)
		});
		methods.add_function("blink", |_, ud: AnyUserData| {
			ud.borrow_mut::<Self>()?.0.add_modifier |= ratatui::style::Modifier::SLOW_BLINK;
			Ok(ud)
		});
		methods.add_function("blink_rapid", |_, ud: AnyUserData| {
			ud.borrow_mut::<Self>()?.0.add_modifier |= ratatui::style::Modifier::RAPID_BLINK;
			Ok(ud)
		});
		methods.add_function("hidden", |_, ud: AnyUserData| {
			ud.borrow_mut::<Self>()?.0.add_modifier |= ratatui::style::Modifier::HIDDEN;
			Ok(ud)
		});
		methods.add_function("crossed", |_, ud: AnyUserData| {
			ud.borrow_mut::<Self>()?.0.add_modifier |= ratatui::style::Modifier::CROSSED_OUT;
			Ok(ud)
		});
		methods.add_function("reset", |_, ud: AnyUserData| {
			ud.borrow_mut::<Self>()?.0.add_modifier = ratatui::style::Modifier::empty();
			Ok(ud)
		});
	}
}
