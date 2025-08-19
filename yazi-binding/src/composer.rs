use hashbrown::HashMap;
use mlua::{Lua, MetaMethod, UserData, UserDataMethods, Value};

pub type ComposerGet = fn(&Lua, &[u8]) -> mlua::Result<Value>;
pub type ComposerSet = fn(&Lua, &[u8], Value) -> mlua::Result<Value>;

pub struct Composer<G, S> {
	get:    G,
	set:    S,
	parent: Option<(G, S)>,
	cache:  HashMap<Vec<u8>, Value>,
}

impl<G, S> Composer<G, S>
where
	G: Fn(&Lua, &[u8]) -> mlua::Result<Value> + 'static,
	S: Fn(&Lua, &[u8], Value) -> mlua::Result<Value> + 'static,
{
	#[inline]
	pub fn new(get: G, set: S) -> Self { Self { get, set, parent: None, cache: Default::default() } }

	#[inline]
	pub fn with_parent(get: G, set: S, p_get: G, p_set: S) -> Self {
		Self { get, set, parent: Some((p_get, p_set)), cache: Default::default() }
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

			let mut value = (me.get)(lua, &key)?;
			if value.is_nil()
				&& let Some((p_get, _)) = &me.parent
			{
				value = p_get(lua, &key)?;
			}

			me.cache.insert(key.to_owned(), value.clone());
			Ok(value)
		});

		methods.add_meta_method_mut(
			MetaMethod::NewIndex,
			|lua, me, (key, value): (mlua::String, Value)| {
				let key = key.as_bytes();
				let value = (me.set)(lua, key.as_ref(), value)?;

				if value.is_nil() {
					me.cache.remove(key.as_ref());
				} else if let Some((_, p_set)) = &me.parent {
					match p_set(lua, key.as_ref(), value)? {
						Value::Nil => me.cache.remove(key.as_ref()),
						v => me.cache.insert(key.to_owned(), v),
					};
				} else {
					me.cache.insert(key.to_owned(), value);
				}

				Ok(())
			},
		);
	}
}
