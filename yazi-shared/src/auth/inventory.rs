use crate::{auth::{AuthArc, Scheme}, domain::Domain};

pub struct AuthInventory {
	pub get: fn(&Scheme, &Domain<'_>) -> Option<AuthArc>,
}

inventory::collect!(AuthInventory);
