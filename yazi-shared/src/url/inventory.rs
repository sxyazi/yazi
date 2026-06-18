use mlua::UserDataRegistry;

use crate::url::UrlBuf;

pub struct UrlBufInventory {
	pub register: fn(&mut UserDataRegistry<UrlBuf>),
}

inventory::collect!(UrlBufInventory);
