use std::{collections::HashSet, ops::Deref};

use yazi_shared::fs::Url;

#[derive(Default)]
pub struct Yanked {
	pub cut:         bool,
	pub(super) urls: HashSet<Url>,
}

impl Deref for Yanked {
	type Target = HashSet<Url>;

	fn deref(&self) -> &Self::Target { &self.urls }
}
