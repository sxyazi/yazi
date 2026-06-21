use mlua::UserDataRegistry;

use crate::file::File;

pub struct FileInventory {
	pub register: fn(&mut UserDataRegistry<File>),
}

inventory::collect!(FileInventory);
