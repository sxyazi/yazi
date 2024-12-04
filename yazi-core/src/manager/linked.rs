use std::{collections::HashMap, ops::{Deref, DerefMut}};

use yazi_shared::url::Url;

#[derive(Default)]
pub struct Linked(HashMap<Url, Url> /* from ==> to */);

impl Deref for Linked {
	type Target = HashMap<Url, Url>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl DerefMut for Linked {
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl Linked {
	pub fn from_dir<'a, 'b>(&'a self, url: &'b Url) -> Box<dyn Iterator<Item = &'a Url> + 'b>
	where
		'a: 'b,
	{
		if let Some(to) = self.get(url) {
			Box::new(self.iter().filter(move |(k, v)| *v == to && *k != url).map(|(k, _)| k))
		} else {
			Box::new(self.iter().filter(move |(_, v)| *v == url).map(|(k, _)| k))
		}
	}

	pub fn from_file(&self, url: &Url) -> Vec<Url> {
		if self.is_empty() {
			return vec![];
		}

		let Some(p) = url.parent_url() else {
			return vec![];
		};

		let name = url.file_name().unwrap();
		self.from_dir(&p).map(|u| u.join(name)).collect()
	}
}
