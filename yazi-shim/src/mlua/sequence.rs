use std::marker::PhantomData;

use mlua::{FromLua, Integer, IntoLua, Lua, Table, ffi};

// mlua's `TableSequence<'_, V>` holds `table: &'a Table` and cannot be Clone.
// By owning both `Lua` and `Table` (both are reference-counted; Clone = two
// atomic increments) we get a Clone-able sequence iterator with zero heap
// allocation.
//
// `next()` acquires the Lua lock once per element (same as `raw_get`), but uses
// `lua_rawgeti` to skip the separate integer key push that `raw_get` requires.
// Holding the lock across all elements (as `TableSequence` does) is not
// possible here because `LuaGuard` is `pub(crate)` and cannot be stored in
// Clone types outside mlua.
#[derive(Clone)]
pub struct SequenceIter<V> {
	pub(super) lua:      Lua,
	pub(super) table:    Table,
	pub(super) index:    Integer,
	pub(super) _phantom: PhantomData<V>,
}

impl<V: FromLua> Iterator for SequenceIter<V> {
	type Item = mlua::Result<V>;

	fn next(&mut self) -> Option<mlua::Result<V>> {
		self.index += 1;
		self.lua.exec_raw_lua(|lua| {
			unsafe {
				let state = lua.state();
				if ffi::lua_checkstack(state, 2) == 0 {
					return Some(Err(mlua::Error::StackError));
				}

				if let Err(e) = <&Table as IntoLua>::push_into_stack(&self.table, lua) {
					return Some(Err(e));
				};

				let result = match ffi::lua_rawgeti(state, -1, self.index) {
					ffi::LUA_TNIL => None,
					_ => Some(<V as FromLua>::from_stack(-1, lua)),
				};

				ffi::lua_pop(state, 2); // table + value
				result
			}
		})
	}
}
