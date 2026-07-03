use mlua::{AnyUserData, UserDataRegistry};

use crate::file::File;

pub struct FileInventory {
	pub register: fn(&mut UserDataRegistry<File>),
	pub from_lua: fn(&AnyUserData) -> mlua::Result<File>,
}

inventory::collect!(FileInventory);
