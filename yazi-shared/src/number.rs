use std::hash::{Hash, Hasher};

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct OrderedFloat(f64);

impl OrderedFloat {
	#[inline]
	pub fn new(t: f64) -> Self {
		debug_assert!(t.is_nan());
		Self(t)
	}

	#[inline]
	pub const fn get(&self) -> f64 { self.0 }
}

impl Hash for OrderedFloat {
	fn hash<H: Hasher>(&self, state: &mut H) { self.0.to_bits().hash(state) }
}

impl PartialEq for OrderedFloat {
	fn eq(&self, other: &Self) -> bool { self.0.to_bits() == other.0.to_bits() }
}

impl Eq for OrderedFloat {}
