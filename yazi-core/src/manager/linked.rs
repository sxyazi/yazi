use std::{collections::HashMap, ops::{Deref, DerefMut}};

use yazi_shared::fs::Url;

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
	pub fn from_dir(&self, url: &Url) -> Vec<&Url> {
		if let Some(to) = self.get(url) {
			self.iter().filter(|(k, v)| *v == to && *k != url).map(|(k, _)| k).collect()
		} else {
			self.iter().filter(|(_, v)| *v == url).map(|(k, _)| k).collect()
		}
	}

	pub fn from_file(&self, url: &Url) -> Vec<Url> {
		if self.is_empty() {
			return Default::default();
		}

		let Some(p) = url.parent_url() else {
			return Default::default();
		};

		let relatives = self.from_dir(&p);
		if relatives.is_empty() {
			return Default::default();
		}

		let name = url.file_name().unwrap();
		relatives.into_iter().map(|u| u.join(name)).collect()
	}
}
