use std::{collections::HashSet, ops::{Deref, DerefMut}};

use yazi_shared::fs::{FilesOp, Url};

#[derive(Default)]
pub struct Yanked {
	pub cut:         bool,
	pub(super) urls: HashSet<Url>,
}

impl Deref for Yanked {
	type Target = HashSet<Url>;

	fn deref(&self) -> &Self::Target { &self.urls }
}

impl DerefMut for Yanked {
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.urls }
}

impl Yanked {
	pub fn apply_op(&mut self, op: &FilesOp) {
		let (removal, addition) = match op {
			FilesOp::Deleting(_, urls) => (urls.iter().collect(), vec![]),
			FilesOp::Updating(_, urls) | FilesOp::Upserting(_, urls) => {
				urls.iter().filter(|(u, _)| self.contains(u)).map(|(u, f)| (u, f.url())).unzip()
			}
			_ => (vec![], vec![]),
		};

		self.urls.retain(|u| !removal.contains(&u));
		self.urls.extend(addition);
	}
}
