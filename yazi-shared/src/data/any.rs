use std::{any::{self, Any, TypeId}, fmt::{self, Debug}};

use dyn_clone::DynClone;
use mlua::{IntoLua, Lua, Value};

use crate::any_data::AnyData;

pub trait DataAny: Any + Send + Sync + DynClone {
	fn as_any(&self, id: TypeId) -> Option<&dyn Any>;

	fn into_any(self: Box<Self>, id: TypeId) -> Result<Box<dyn Any>, Box<dyn DataAny>>;

	fn into_lua(self: Box<Self>, lua: &Lua) -> mlua::Result<Value>;

	fn to_lua(&self, lua: &Lua) -> mlua::Result<Value> { dyn_clone::clone_box(self).into_lua(lua) }

	fn from_lua(_value: Value, _lua: &Lua) -> mlua::Result<Box<dyn DataAny>>
	where
		Self: Sized,
	{
		Err(mlua::Error::runtime(format!(
			"DataAny::from_lua is not implemented for `{}`",
			any::type_name::<Self>(),
		)))
	}
}

impl dyn DataAny {
	pub fn downcast_ref<T: 'static>(&self) -> Option<&T> {
		self.as_any(TypeId::of::<T>()).and_then(|a| a.downcast_ref::<T>())
	}

	pub fn downcast<T: 'static>(self: Box<Self>) -> Result<Box<T>, Box<dyn Any>> {
		let id = TypeId::of::<T>();
		self.into_any(id).map_or_else(|me| Err(me as Box<dyn Any>), |a| a.downcast::<T>())
	}
}

impl Clone for Box<dyn DataAny> {
	fn clone(&self) -> Self { dyn_clone::clone_box(&**self) }
}

impl Debug for dyn DataAny {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("DataAny").finish_non_exhaustive()
	}
}

impl<T: Clone + Send + Sync + 'static> DataAny for Vec<T> {
	fn as_any(&self, id: TypeId) -> Option<&dyn Any> {
		(id == TypeId::of::<Self>()).then_some(self as &dyn Any)
	}

	fn into_any(self: Box<Self>, id: TypeId) -> Result<Box<dyn Any>, Box<dyn DataAny>> {
		if id == TypeId::of::<Self>() { Ok(self) } else { Err(self) }
	}

	fn into_lua(self: Box<Self>, lua: &Lua) -> mlua::Result<Value> { AnyData(self).into_lua(lua) }
}

impl<T: Send + 'static> DataAny for tokio::sync::mpsc::UnboundedSender<T> {
	fn as_any(&self, id: TypeId) -> Option<&dyn Any> {
		(id == TypeId::of::<Self>()).then_some(self as &dyn Any)
	}

	fn into_any(self: Box<Self>, id: TypeId) -> Result<Box<dyn Any>, Box<dyn DataAny>> {
		if id == TypeId::of::<Self>() { Ok(self) } else { Err(self) }
	}

	fn into_lua(self: Box<Self>, lua: &Lua) -> mlua::Result<Value> { AnyData(self).into_lua(lua) }
}
