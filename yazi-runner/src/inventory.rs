use mlua::Lua;

pub struct RunnerInventory {
	pub setup: fn(&Lua) -> mlua::Result<()>,
}

inventory::collect!(RunnerInventory);
