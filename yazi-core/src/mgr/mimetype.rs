use hashbrown::HashMap;
use yazi_shared::{pool::Symbol, url::{Url, UrlBufCov, UrlCov}};

#[derive(Default)]
pub struct Mimetype(HashMap<UrlBufCov, Symbol<str>>);

impl Mimetype {
	pub fn get<'a, 'b>(&'a self, url: impl Into<Url<'b>>) -> Option<&'a str> {
		self.0.get(&UrlCov::new(url)).map(|s| s.as_ref())
	}

	pub fn owned<'a>(&self, url: impl Into<Url<'a>>) -> Option<Symbol<str>> {
		self.0.get(&UrlCov::new(url)).cloned()
	}

	pub fn contains<'a>(&self, url: impl Into<Url<'a>>) -> bool {
		self.0.contains_key(&UrlCov::new(url))
	}

	pub fn extend(&mut self, iter: impl IntoIterator<Item = (UrlBufCov, Symbol<str>)>) {
		self.0.extend(iter);
	}
}
