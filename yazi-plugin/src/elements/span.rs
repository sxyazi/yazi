use mlua::{AnyUserData, ExternalError, FromLua, Lua, Table, UserData, UserDataMethods, Value};
use yazi_config::theme::Color;

use super::Style;

#[derive(Clone, FromLua)]
pub struct Span(pub(super) ratatui::text::Span<'static>);

impl Span {
	pub fn install(lua: &Lua, ui: &Table) -> mlua::Result<()> {
		ui.set(
			"Span",
			lua.create_function(|_, content: mlua::String| {
				Ok(Self(ratatui::text::Span::raw(content.to_string_lossy().into_owned())))
			})?,
		)
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
				Value::Table(tb) => Style::from(tb).0,
				Value::UserData(ud) => ud.borrow::<Style>()?.0,
				_ => return Err("expected a Style or Table or nil".into_lua_err()),
			};
			Ok(ud)
		});
	}
}
