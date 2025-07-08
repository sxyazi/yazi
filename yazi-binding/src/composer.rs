use foldhash::HashMap;
use mlua::{IntoLua, Lua, MetaMethod, UserData, UserDataMethods, Value};

pub struct Composer<G, S> {
	get:   G,
	set:   S,
	cache: HashMap<Vec<u8>, Value>,
}

impl<G, S> Composer<G, S>
where
	G: Fn(&Lua, &[u8]) -> mlua::Result<Value> + 'static,
	S: Fn(&Lua, &[u8], Value) -> mlua::Result<Value> + 'static,
{
	pub fn make(lua: &Lua, get: G, set: S) -> mlua::Result<Value> {
		Self { get, set, cache: Default::default() }.into_lua(lua)
	}
}

impl<G, S> UserData for Composer<G, S>
where
	G: Fn(&Lua, &[u8]) -> mlua::Result<Value> + 'static,
	S: Fn(&Lua, &[u8], Value) -> mlua::Result<Value> + 'static,
{
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_meta_method_mut(MetaMethod::Index, |lua, me, key: mlua::String| {
			let key = key.as_bytes();
			if let Some(v) = me.cache.get(key.as_ref()) {
				return Ok(v.clone());
			}

			let v = (me.get)(lua, &key)?;
			me.cache.insert(key.to_owned(), v.clone());
			Ok(v)
		});

		methods.add_meta_method_mut(
			MetaMethod::NewIndex,
			|lua, me, (key, value): (mlua::String, Value)| {
				let key = key.as_bytes();

				let value = (me.set)(lua, key.as_ref(), value)?;
				if value.is_nil() {
					me.cache.remove(key.as_ref());
				} else {
					me.cache.insert(key.to_owned(), value);
				}

				Ok(())
			},
		);
	}
}
