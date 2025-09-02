use mlua::{AnyUserData, ExternalError, FromLua, IntoLua, IntoLuaMulti, Lua, MetaMethod, UserData, UserDataMethods, UserDataRefMut, Value};

pub struct Iter<I: Iterator<Item = T>, T> {
	iter: I,
	len:  Option<usize>,

	count: usize,
	cache: Vec<Value>,
}

impl<I, T> Iter<I, T>
where
	I: Iterator<Item = T> + 'static,
	T: IntoLua + 'static,
{
	pub fn new(iter: I, len: Option<usize>) -> Self { Self { iter, len, count: 0, cache: vec![] } }
}

impl<I, T> Iter<I, T>
where
	I: Iterator<Item = T> + 'static,
	T: FromLua + 'static,
{
	pub fn into_iter(self, lua: &Lua) -> impl Iterator<Item = mlua::Result<T>> {
		self
			.cache
			.into_iter()
			.map(|cached| T::from_lua(cached, lua))
			.chain(self.iter.map(|rest| Ok(rest)))
	}
}

impl<I, T> UserData for Iter<I, T>
where
	I: Iterator<Item = T> + 'static,
	T: IntoLua + 'static,
{
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_meta_method(MetaMethod::Len, |_, me, ()| {
			if let Some(len) = me.len {
				Ok(len)
			} else {
				Err(format!("Length is unknown for {}", std::any::type_name::<Self>()).into_lua_err())
			}
		});

		methods.add_meta_function(MetaMethod::Pairs, |lua, ud: AnyUserData| {
			let iter = lua.create_function(|lua, mut me: UserDataRefMut<Self>| {
				if let Some(next) = me.cache.get(me.count).cloned() {
					me.count += 1;
					(me.count, next).into_lua_multi(lua)
				} else if let Some(next) = me.iter.next() {
					let value = next.into_lua(lua)?;
					me.cache.push(value.clone());
					me.count += 1;
					(me.count, value).into_lua_multi(lua)
				} else {
					().into_lua_multi(lua)
				}
			})?;

			ud.borrow_mut::<Self>()?.count = 0;
			Ok((iter, ud))
		});
	}
}
