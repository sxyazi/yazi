use mlua::{AnyUserData, Lua, Table, UserData, UserDataMethods};
use yazi_config::theme::Color;

#[derive(Clone, Copy, Default)]
pub struct Style(pub(super) ratatui::style::Style);

impl Style {
	pub fn install(lua: &Lua, ui: &Table) -> mlua::Result<()> {
		let new = lua.create_function(|_, ()| Ok(Self::default()))?;

		let style = lua.create_table()?;
		style.set_metatable(Some(lua.create_table_from([("__call", new)])?));

		ui.set("Style", style)
	}
}

impl From<yazi_config::theme::Style> for Style {
	fn from(value: yazi_config::theme::Style) -> Self { Self(value.into()) }
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
		style.add_modifier =
			ratatui::style::Modifier::from_bits_truncate(value.get("modifier").unwrap_or_default());
		Self(style)
	}
}

impl UserData for Style {
	fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
		methods.add_function("fg", |_, (ud, color): (AnyUserData, String)| {
			ud.borrow_mut::<Self>()?.0.fg = Color::try_from(color).ok().map(Into::into);
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
