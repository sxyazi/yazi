use std::any::{Any, TypeId};

use mlua::{AnyUserData, ExternalError, Function, Lua};
use parking_lot::Mutex;
use yazi_binding::process::Child;

use super::Utils;

pub(super) static HELD: Mutex<Vec<Box<dyn Any + Send>>> = Mutex::new(Vec::new());

impl Utils {
	pub(super) fn hold(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|_, ud: AnyUserData| {
			let t = match ud.type_id() {
				Some(t) if t == TypeId::of::<Child>() => Box::new(ud.take::<Child>()?),
				Some(t) => Err(format!("Cannot hold userdata of type {t:?}").into_lua_err())?,
				None => Err("Cannot hold scoped userdata".into_lua_err())?,
			};

			Ok(HELD.lock().push(t))
		})
	}
}
