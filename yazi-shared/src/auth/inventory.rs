use std::sync::Arc;

use crate::auth::{Auth, Domain, Scheme};

pub struct AuthInventory {
	pub get: fn(&Scheme, &Domain<'_>) -> Option<Arc<Auth>>,
}

inventory::collect!(AuthInventory);
