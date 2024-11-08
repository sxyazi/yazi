use mlua::{FromLua, Lua, Table, UserData};

#[derive(Clone, Copy, Default, FromLua)]
pub struct Constraint(pub(super) ratatui::layout::Constraint);

impl Constraint {
	pub fn install(_: &Lua, ui: &Table) -> mlua::Result<()> {
		ui.raw_set("Constraint", Constraint::default())
	}
}

impl From<Constraint> for ratatui::layout::Constraint {
	fn from(value: Constraint) -> Self { value.0 }
}

impl UserData for Constraint {
	fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
		use ratatui::layout::Constraint as C;

		methods.add_function("Min", |_, n: u16| Ok(Self(C::Min(n))));
		methods.add_function("Max", |_, n: u16| Ok(Self(C::Max(n))));
		methods.add_function("Length", |_, n: u16| Ok(Self(C::Length(n))));
		methods.add_function("Percentage", |_, n: u16| Ok(Self(C::Percentage(n))));
		methods.add_function("Ratio", |_, (a, b): (u32, u32)| Ok(Self(C::Ratio(a, b))));
		methods.add_function("Fill", |_, n: u16| Ok(Self(C::Fill(n))));
	}
}
