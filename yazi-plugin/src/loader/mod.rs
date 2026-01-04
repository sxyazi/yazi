yazi_macro::mod_flat!(chunk loader require);

pub(super) fn init() { LOADER.with(<_>::default); }

pub(super) fn install(lua: &mlua::Lua) -> mlua::Result<()> { Require::install(lua) }
