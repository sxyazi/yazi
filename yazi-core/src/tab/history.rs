use std::ops::{Deref, DerefMut};

use hashbrown::HashMap;
use yazi_shared::url::{AsUrl, UrlBuf};

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
	pub fn remove_or(&mut self, url: impl AsUrl) -> Folder {
		let url = url.as_url();
		self.0.remove(&url).unwrap_or_else(|| Folder::from(url))
	}
}
