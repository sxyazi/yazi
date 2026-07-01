#[macro_export]
macro_rules! impl_data_any {
	($ty:ty) => {
		impl ::yazi_shared::data::DataAny for $ty {
			$crate::impl_data_any!(@core);
			$crate::impl_data_any!(@into_lua_default);
		}
	};
	($ty:ty => $($target:ty),+) => {
		impl ::yazi_shared::data::DataAny for $ty {
			$crate::impl_data_any!(@core $($target),+);
			$crate::impl_data_any!(@into_lua_default);
		}
	};
	// --- from_into_lua
	($ty:ty, from_into_lua = inherit) => {
		impl ::yazi_shared::data::DataAny for $ty {
			$crate::impl_data_any!(@core);
			$crate::impl_data_any!(@from_lua_inherit);
			$crate::impl_data_any!(@into_lua_inherit);
		}
		$crate::impl_data_any!(@from_lua_register $ty);
	};
	($ty:ty => $($target:ty),+ ; from_into_lua = inherit) => {
		impl ::yazi_shared::data::DataAny for $ty {
			$crate::impl_data_any!(@core $($target),+);
			$crate::impl_data_any!(@from_lua_inherit);
			$crate::impl_data_any!(@into_lua_inherit);
		}
		$crate::impl_data_any!(@from_lua_register $ty);
	};
	// --- to_lua
	($ty:ty, to_lua = $body:expr) => {
		impl ::yazi_shared::data::DataAny for $ty {
			$crate::impl_data_any!(@core);
			$crate::impl_data_any!(@from_lua_inherit);
			$crate::impl_data_any!(@into_lua_default);
			$crate::impl_data_any!(@to_lua $body);
		}
		$crate::impl_data_any!(@from_lua_register $ty);
	};
	($ty:ty => $($target:ty),+ ; to_lua = $body:expr) => {
		impl ::yazi_shared::data::DataAny for $ty {
			$crate::impl_data_any!(@core $($target),+);
			$crate::impl_data_any!(@from_lua_inherit);
			$crate::impl_data_any!(@into_lua_default);
			$crate::impl_data_any!(@to_lua $body);
		}
		$crate::impl_data_any!(@from_lua_register $ty);
	};

	(@core $($target:ty),*) => {
		fn as_any(&self, id: std::any::TypeId) -> Option<&dyn std::any::Any> {
			use std::any::{TypeId, Any};

			if id == TypeId::of::<Self>() {
				return Some(self as &dyn Any);
			}
			$(
				if id == TypeId::of::<$target>() {
					return Some(<Self as AsRef<$target>>::as_ref(self) as &dyn Any);
				}
			)*
			None
		}

		fn into_any(self: Box<Self>, id: std::any::TypeId) -> Result<Box<dyn std::any::Any>, Box<dyn ::yazi_shared::data::DataAny>> {
			use std::any::TypeId;

			if id == TypeId::of::<Self>() { return Ok(self); }
			$(
				if id == TypeId::of::<$target>() {
					return Ok(Box::new(<Self as Into<$target>>::into(*self)));
				}
			)*
			Err(self)
		}
	};
	(@into_lua_default) => {
		fn into_lua(self: Box<Self>, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
			use ::yazi_shared::any_data::AnyData;
			use mlua::IntoLua;

			AnyData(self).into_lua(lua)
		}
	};
	(@into_lua_inherit) => {
		fn into_lua(self: Box<Self>, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
			use mlua::IntoLua;

			IntoLua::into_lua(*self, lua)
		}
	};
	(@to_lua $body:expr) => {
		fn to_lua(&self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
			use mlua::{self, Lua, Value};

			($body as fn(&Self, &Lua) -> mlua::Result<Value>)(self, lua)
		}
	};
	(@from_lua_inherit) => {
		fn from_lua(value: &mlua::Value, lua: &mlua::Lua) -> mlua::Result<Box<dyn ::yazi_shared::data::DataAny>> {
			use mlua::FromLua;

			<Self as FromLua>::from_lua(value.clone(), lua)
				.map(|v| Box::new(v) as Box<dyn ::yazi_shared::data::DataAny>)
		}
	};
	(@from_lua_register $ty:ty) => {
		::inventory::submit! {
			::yazi_shared::data::DataInventory {
				from_lua: <$ty as ::yazi_shared::data::DataAny>::from_lua,
			}
		}
	};
}
