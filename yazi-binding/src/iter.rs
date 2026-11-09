use mlua::{AnyUserData, FromLua, IntoLua, IntoLuaMulti, Lua, MetaMethod, UserData, UserDataMethods, UserDataRefMut, Value};

pub struct Iter<I> {
	iter:  I,
	len:   usize,
	cache: Vec<Value>,
}

impl<I> Iter<I> {
	pub fn new(iter: I, len: usize) -> Self { Self { iter, len, cache: vec![] } }
}

impl<I> Iter<I>
where
	I: Iterator,
	I::Item: FromLua,
{
	pub fn into_iter(self, lua: &Lua) -> impl Iterator<Item = mlua::Result<I::Item>> {
		self.cache.into_iter().map(|cached| I::Item::from_lua(cached, lua)).chain(self.iter.map(Ok))
	}
}

impl<I> UserData for Iter<I>
where
	I: Iterator + 'static,
	I::Item: IntoLua,
{
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_meta_method(MetaMethod::Len, |_, me, ()| Ok(me.len));

		methods.add_meta_function(MetaMethod::Pairs, |lua, ud: AnyUserData| {
			let iter = lua.create_function(|lua, (mut me, index): (UserDataRefMut<Self>, usize)| {
				if let Some(next) = me.cache.get(index).cloned() {
					(index + 1, next).into_lua_multi(lua)
				} else if let Some(next) = me.iter.next() {
					let value = next.into_lua(lua)?;
					me.cache.push(value.clone());
					(index + 1, value).into_lua_multi(lua)
				} else {
					().into_lua_multi(lua)
				}
			})?;

			Ok((iter, ud, 0))
		});
	}
}
