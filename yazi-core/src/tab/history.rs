use std::ops::{Deref, DerefMut};

use hashbrown::HashMap;
use yazi_shared::url::{Url, UrlBuf};

use super::Folder;

#[derive(Default)]
pub struct History(HashMap<UrlBuf, Folder>);

impl Deref for History {
	type Target = HashMap<UrlBuf, Folder>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl DerefMut for History {
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl History {
	pub fn remove_or<'a>(&mut self, url: impl Into<Url<'a>>) -> Folder {
		let url = url.into();
		self.0.remove(&url).unwrap_or_else(|| Folder::from(url))
	}
}
