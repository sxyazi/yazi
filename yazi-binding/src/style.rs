use mlua::{AnyUserData, ExternalError, IntoLua, Lua, MetaMethod, Table, UserData, UserDataMethods, Value};

#[derive(Clone, Copy, Default)]
pub struct Style(pub ratatui::style::Style);

impl Style {
	pub fn compose(lua: &Lua) -> mlua::Result<Value> {
		let new = lua.create_function(|_, (_, value): (Table, Value)| Self::try_from(value))?;

		let style = lua.create_table()?;
		style.set_metatable(Some(lua.create_table_from([(MetaMethod::Call.name(), new)])?));

		style.into_lua(lua)
	}
}

impl TryFrom<Value> for Style {
	type Error = mlua::Error;

	fn try_from(value: Value) -> Result<Self, Self::Error> {
		Ok(Self(match value {
			Value::Nil => Default::default(),
			Value::UserData(ud) => ud.borrow::<Self>()?.0,
			_ => Err("expected a Style or nil".into_lua_err())?,
		}))
	}
}

impl From<yazi_config::Style> for Style {
	fn from(value: yazi_config::Style) -> Self { Self(value.into()) }
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
