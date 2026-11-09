use mlua::{BorrowedBytes, Function, IntoLua, Lua, Value};
use yazi_shim::cached_chunk;

pub type ComposerGet = fn(&Lua, &[u8]) -> mlua::Result<Value>;
pub type ComposerSet = fn(&Lua, &[u8], Value) -> mlua::Result<Value>;

pub struct Composer<G> {
	get: G,
	set: Option<ComposerSet>,
}

impl<G> Composer<G>
where
	G: Fn(&Lua, &[u8]) -> mlua::Result<Value> + 'static,
{
	pub fn new(get: G) -> Self { Self { get, set: None } }

	pub fn with_setter(get: G, set: ComposerSet) -> Self { Self { get, set: Some(set) } }
}

impl<G> IntoLua for Composer<G>
where
	G: Fn(&Lua, &[u8]) -> mlua::Result<Value> + 'static,
{
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		let f: Function = match lua.named_registry_value("composer")? {
			Some(f) => f,
			None => {
				let f = cached_chunk!(lua, "composer.lua", include_str!("composer.lua"))?;
				lua.set_named_registry_value("composer", &f)?;
				f
			}
		};

		let get = lua.create_function(move |lua, key: BorrowedBytes| (self.get)(lua, &key))?;
		let set = self
			.set
			.map(|set| {
				lua.create_function(move |lua, (key, value): (BorrowedBytes, Value)| set(lua, &key, value))
			})
			.transpose()?;

		f.call((get, set))
	}
}
