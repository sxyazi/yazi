use hashbrown::HashMap;
use yazi_fs::File;
use yazi_shared::{MIME_DIR, pool::{InternStr, Symbol}, url::{Url, UrlBufCov, UrlCov}};

#[derive(Default)]
pub struct Mimetype(HashMap<UrlBufCov, Symbol<str>>);

impl Mimetype {
	pub fn by_url<'a>(&self, url: impl Into<Url<'a>>) -> Option<&str> {
		self.0.get(&UrlCov::new(url)).map(|s| s.as_ref())
	}

	pub fn by_url_owned<'a>(&self, url: impl Into<Url<'a>>) -> Option<Symbol<str>> {
		self.0.get(&UrlCov::new(url)).cloned()
	}

	pub fn by_file(&self, file: &File) -> Option<&str> {
		if file.is_dir() { Some(MIME_DIR) } else { self.by_url(&file.url) }
	}

	pub fn by_file_owned(&self, file: &File) -> Option<Symbol<str>> {
		if file.is_dir() { Some(MIME_DIR.intern()) } else { self.by_url_owned(&file.url) }
	}

	pub fn contains<'a>(&self, url: impl Into<Url<'a>>) -> bool {
		self.0.contains_key(&UrlCov::new(url))
	}

	pub fn extend(&mut self, iter: impl IntoIterator<Item = (UrlBufCov, Symbol<str>)>) {
		self.0.extend(iter);
	}
}
