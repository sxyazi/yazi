use std::{collections::HashMap, ops::{Deref, DerefMut}};

use yazi_shared::url::Url;

use super::Folder;

#[derive(Default)]
pub struct History(HashMap<Url, Folder>);

impl Deref for History {
	type Target = HashMap<Url, Folder>;

	#[inline]
	fn deref(&self) -> &Self::Target { &self.0 }
}

impl DerefMut for History {
	#[inline]
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl History {
	#[inline]
	pub fn remove_or(&mut self, url: &Url) -> Folder {
		self.0.remove(url).unwrap_or_else(|| Folder::from(url))
	}
}
