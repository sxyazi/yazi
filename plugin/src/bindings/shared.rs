use mlua::{MetaMethod, UserData};

pub struct Url(shared::Url);

impl From<&shared::Url> for Url {
	fn from(value: &shared::Url) -> Self { Self(value.clone()) }
}

impl UserData for Url {
	fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
		methods.add_meta_method(MetaMethod::ToString, |_, me, ()| Ok(me.0.display().to_string()));
	}
}
