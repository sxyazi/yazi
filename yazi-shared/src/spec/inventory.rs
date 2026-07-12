use mlua::UserDataRegistry;

use crate::spec::Spec;

pub struct SpecInventory {
	pub register: fn(&mut UserDataRegistry<Spec>),
}

inventory::collect!(SpecInventory);
