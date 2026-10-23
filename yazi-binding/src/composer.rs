use std::sync::OnceLock;

use mlua::{BorrowedBytes, Function, IntoLua, Lua, Value, chunk::ChunkMode};

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

	fn load(lua: &Lua) -> mlua::Result<Function> {
		static CACHE: OnceLock<Box<[u8]>> = OnceLock::new();

		if let Some(b) = CACHE.get() {
			return lua.load(&b[..]).set_mode(ChunkMode::Binary).set_name("composer.lua").into_function();
		}

		let f = lua.load(include_str!("composer.lua")).set_name("composer.lua").into_function()?;
		_ = CACHE.set(f.dump(false).into_boxed_slice());
		Ok(f)
	}
}

impl<G> IntoLua for Composer<G>
where
	G: Fn(&Lua, &[u8]) -> mlua::Result<Value> + 'static,
{
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		let f: Function = match lua.named_registry_value("composer")? {
			Some(f) => f,
			None => {
				let f = Self::load(lua)?;
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
