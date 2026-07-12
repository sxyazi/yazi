use std::sync::Arc;

use crate::auth::{Auth, Scheme};

pub struct AuthInventory {
	pub get: fn(&Scheme, &str) -> Option<Arc<Auth>>,
}

inventory::collect!(AuthInventory);
