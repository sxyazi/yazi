use std::sync::OnceLock;

use mlua::{Function, Lua, chunk::ChunkMode};

pub struct ChunkCache {
	name:  &'static str,
	bytes: OnceLock<Box<[u8]>>,
}

impl ChunkCache {
	pub const fn new(name: &'static str) -> Self { Self { name, bytes: OnceLock::new() } }

	pub fn load<F, B>(&self, lua: &Lua, source: F) -> mlua::Result<Function>
	where
		F: FnOnce() -> B,
		B: AsRef<[u8]>,
	{
		if let Some(bytes) = self.bytes.get() {
			return lua.load(&bytes[..]).set_mode(ChunkMode::Binary).set_name(self.name).into_function();
		}

		let f = lua.load(source().as_ref()).set_name(self.name).into_function()?;
		_ = self.bytes.set(f.dump(false).into_boxed_slice());
		Ok(f)
	}
}

/// Caches a fixed chunk per call site, evaluating its source only on a cache miss.
#[macro_export]
macro_rules! cached_chunk {
	($lua:expr, $name:expr, $source:expr $(,)?) => {{
		static CACHE: $crate::mlua::ChunkCache = $crate::mlua::ChunkCache::new($name);
		CACHE.load($lua, || $source)
	}};
}
