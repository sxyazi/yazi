use std::ops::Deref;

use mlua::{ExternalError, FromLua, Lua, Table, UserData, UserDataFields, UserDataMethods, UserDataRef, Value};
use yazi_binding::Url;

use crate::{bindings::Cha, impl_file_fields, impl_file_methods};

pub type FileRef = UserDataRef<File>;

pub struct File {
	inner: yazi_fs::File,

	v_cha:     Option<Value>,
	v_url:     Option<Value>,
	v_link_to: Option<Value>,
	v_name:    Option<Value>,
}

impl Deref for File {
	type Target = yazi_fs::File;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl From<File> for yazi_fs::File {
	fn from(value: File) -> Self { value.inner }
}

impl File {
	pub fn new(inner: yazi_fs::File) -> Self {
		Self { inner, v_cha: None, v_url: None, v_link_to: None, v_name: None }
	}
}

impl File {
	pub fn install(lua: &Lua) -> mlua::Result<()> {
		lua.globals().raw_set(
			"File",
			lua.create_function(|_, t: Table| {
				Ok(Self::new(yazi_fs::File {
					url: t.raw_get::<Url>("url")?.into(),
					cha: *t.raw_get::<Cha>("cha")?,
					..Default::default()
				}))
			})?,
		)
	}
}

impl FromLua for File {
	fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
		Ok(match value {
			Value::UserData(ud) => Self::new(ud.take::<Self>()?.inner),
			_ => Err("Expected a File".into_lua_err())?,
		})
	}
}

impl UserData for File {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		impl_file_fields!(fields);
	}

	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		impl_file_methods!(methods);
	}
}
