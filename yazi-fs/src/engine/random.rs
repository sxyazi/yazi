use mlua::{ExternalResult, FromLua, Lua, LuaString, MetaMethod, UserData, UserDataFields, UserDataMethods, Value};
use strum::{EnumIs, EnumString, IntoStaticStr};
use yazi_shim::strum::IntoStr;

#[derive(Clone, Copy, Default, EnumIs, EnumString, IntoStaticStr)]
#[strum(serialize_all = "lowercase")]
pub enum Random {
	None,
	Read,
	Write,
	#[default]
	Both,
}

impl Random {
	pub fn read(self) -> bool { matches!(self, Self::Read | Self::Both) }

	pub fn write(self) -> bool { matches!(self, Self::Write | Self::Both) }
}

impl FromLua for Random {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		match value {
			Value::Nil => Ok(Self::default()),
			Value::UserData(ud) => Ok(*ud.borrow::<Self>()?),
			v => LuaString::from_lua(v, lua)?.to_str()?.parse().into_lua_err(),
		}
	}
}

impl UserData for Random {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("read", |_, me| Ok(me.read()));
		fields.add_field_method_get("write", |_, me| Ok(me.write()));
	}

	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_meta_method(MetaMethod::ToString, |_, me, ()| Ok(me.into_str()));
	}
}
