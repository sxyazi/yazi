use mlua::{AnyUserData, BorrowedBytes, Function, Lua, MetaMethod};
use yazi_macro::plugin_preset as preset;
use yazi_shim::cached_chunk;

use super::Utils;

impl Utils {
	pub(super) fn iter(lua: &Lua) -> mlua::Result<Function> {
		let load = lua.create_function(|lua, key: BorrowedBytes| {
			Ok(Some(match &*key {
				b"_array" => cached_chunk!(lua, "iter.array.lua", preset!("iter/array"))?,
				b"all" => cached_chunk!(lua, "iter.all.lua", preset!("iter/all"))?,
				b"any" => cached_chunk!(lua, "iter.any.lua", preset!("iter/any"))?,
				b"each" => cached_chunk!(lua, "iter.each.lua", preset!("iter/each"))?,
				b"enumerate" => cached_chunk!(lua, "iter.enumerate.lua", preset!("iter/enumerate"))?,
				b"filter" => cached_chunk!(lua, "iter.filter.lua", preset!("iter/filter"))?,
				b"find" => cached_chunk!(lua, "iter.find.lua", preset!("iter/find"))?,
				b"flat_map" => cached_chunk!(lua, "iter.flat_map.lua", preset!("iter/flat_map"))?,
				b"fold" => cached_chunk!(lua, "iter.fold.lua", preset!("iter/fold"))?,
				b"into" => cached_chunk!(lua, "iter.into.lua", preset!("iter/into"))?,
				b"kv" => cached_chunk!(lua, "iter.kv.lua", preset!("iter/kv"))?,
				b"map" => cached_chunk!(lua, "iter.map.lua", preset!("iter/map"))?,
				b"pack" => cached_chunk!(lua, "iter.pack.lua", preset!("iter/pack"))?,
				b"skip" => cached_chunk!(lua, "iter.skip.lua", preset!("iter/skip"))?,
				b"take" => cached_chunk!(lua, "iter.take.lua", preset!("iter/take"))?,
				b"take_while" => cached_chunk!(lua, "iter.take_while.lua", preset!("iter/take_while"))?,
				b"try" => cached_chunk!(lua, "iter.try.lua", preset!("iter/try"))?,
				b"unzip" => cached_chunk!(lua, "iter.unzip.lua", preset!("iter/unzip"))?,
				_ => return Ok(None),
			}))
		})?;

		let callable = lua
			.create_function(|_, ud: AnyUserData| ud.metatable()?.contains(MetaMethod::Call.name()))?;

		cached_chunk!(lua, "iter.lua", preset!("iter/iter"))?.call((load, callable))
	}
}
