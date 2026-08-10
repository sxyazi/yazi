use mlua::UserDataRegistry;

use super::HttpResponse;

pub struct HttpInventory {
	pub register: fn(&mut UserDataRegistry<HttpResponse>),
}

inventory::collect!(HttpInventory);
