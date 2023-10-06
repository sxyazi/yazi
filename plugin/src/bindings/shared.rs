use mlua::{IntoLua, MetaMethod, UserData, UserDataRef};

// --- Range
pub struct Range<T>(std::ops::Range<T>);

impl<T> From<std::ops::Range<T>> for Range<T> {
	fn from(value: std::ops::Range<T>) -> Self { Self(value) }
}

impl<'lua, T> IntoLua<'lua> for Range<T>
where
	T: IntoLua<'lua>,
{
	fn into_lua(self, lua: &'lua mlua::Lua) -> mlua::Result<mlua::Value> {
		let tbl = lua.create_sequence_from([self.0.start, self.0.end])?;
		tbl.into_lua(lua)
	}
}

// --- Url
pub struct Url(shared::Url);

impl From<&shared::Url> for Url {
	fn from(value: &shared::Url) -> Self { Self(value.clone()) }
}

impl UserData for Url {
	fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
		methods.add_meta_function(
			MetaMethod::Eq,
			|_, (lhs, rhs): (UserDataRef<Self>, UserDataRef<Self>)| Ok(lhs.0 == rhs.0),
		);

		methods.add_meta_method(MetaMethod::ToString, |_, me, ()| Ok(me.0.display().to_string()));
	}
}
