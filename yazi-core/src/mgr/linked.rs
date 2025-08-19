use std::{iter, ops::{Deref, DerefMut}};

use hashbrown::HashMap;
use yazi_shared::url::{Url, UrlBuf};

#[derive(Default)]
pub struct Linked(HashMap<UrlBuf, UrlBuf> /* from ==> to */);

impl Deref for Linked {
	type Target = HashMap<UrlBuf, UrlBuf>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl DerefMut for Linked {
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl Linked {
	pub fn from_dir<'a, 'b>(&'a self, url: &'b UrlBuf) -> Box<dyn Iterator<Item = &'a UrlBuf> + 'b>
	where
		'a: 'b,
	{
		if url.scheme.is_virtual() {
			Box::new(iter::empty())
		} else if let Some(to) = self.get(url) {
			Box::new(self.iter().filter(move |(k, v)| *v == to && *k != url).map(|(k, _)| k))
		} else {
			Box::new(self.iter().filter(move |(_, v)| *v == url).map(|(k, _)| k))
		}
	}

	pub fn from_file(&self, url: Url) -> Vec<UrlBuf> {
		if url.scheme.is_virtual() {
			vec![]
		} else if let Some((parent, urn)) = url.pair() {
			self.from_dir(&parent).map(|u| u.join(&urn)).collect()
		} else {
			vec![]
		}
	}
}
