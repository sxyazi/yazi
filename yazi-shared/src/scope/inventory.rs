use mlua::UserDataRegistry;

use super::Scope;

pub struct ScopeInventory {
	pub register: fn(&mut UserDataRegistry<Scope>),
}

inventory::collect!(ScopeInventory);
