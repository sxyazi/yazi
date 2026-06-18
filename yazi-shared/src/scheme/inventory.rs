use mlua::UserDataRegistry;

use crate::scheme::Scheme;

pub struct SchemeInventory {
	pub register: fn(&mut UserDataRegistry<Scheme>),
}

inventory::collect!(SchemeInventory);
