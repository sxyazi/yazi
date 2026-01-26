use mlua::{AnyUserData, ExternalError, FromLua, IntoLua, Lua, LuaSerdeExt, MetaMethod, Table, UserData, UserDataMethods, Value};

use crate::SER_OPT;

#[derive(Clone, Copy, Default)]
pub struct Style(pub ratatui::style::Style);

impl Style {
	pub fn compose(lua: &Lua) -> mlua::Result<Value> {
		let new = lua.create_function(|_, (_, style): (Table, Self)| Ok(style))?;

		let style = lua.create_table()?;
		style.set_metatable(Some(lua.create_table_from([(MetaMethod::Call.name(), new)])?))?;

		style.into_lua(lua)
	}
}

impl From<yazi_config::Style> for Style {
	fn from(value: yazi_config::Style) -> Self { Self(value.into()) }
}

impl FromLua for Style {
	fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
		Ok(Self(match value {
			Value::Nil => Default::default(),
			Value::UserData(ud) => ud.borrow::<Self>()?.0,
			_ => Err("expected a Style or nil".into_lua_err())?,
		}))
	}
}

impl UserData for Style {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		crate::impl_style_shorthands!(methods, 0);

		methods
			.add_method("raw", |lua, me, ()| lua.to_value_with(&yazi_config::Style::from(me.0), SER_OPT));

		methods.add_function_mut("patch", |_, (ud, style): (AnyUserData, Self)| {
			let mut me = ud.borrow_mut::<Self>()?;
			me.0 = me.0.patch(style.0);
			Ok(ud)
		})
	}
}
