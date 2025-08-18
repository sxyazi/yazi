use std::borrow::Cow;

use hashbrown::HashMap;
use yazi_fs::File;
use yazi_shared::{MIME_DIR, SStr, url::{Url, UrlBufCov, UrlCov}};

#[derive(Default)]
pub struct Mimetype(HashMap<UrlBufCov, String>);

impl Mimetype {
	#[inline]
	pub fn by_url<'a>(&self, url: impl Into<Url<'a>>) -> Option<&str> {
		self.0.get(&UrlCov::new(url)).map(|s| s.as_str())
	}

	#[inline]
	pub fn by_url_owned<'a>(&self, url: impl Into<Url<'a>>) -> Option<SStr> {
		self.by_url(url).map(|s| Cow::Owned(s.to_owned()))
	}

	#[inline]
	pub fn by_file(&self, file: &File) -> Option<&str> {
		if file.is_dir() { Some(MIME_DIR) } else { self.by_url(&file.url) }
	}

	#[inline]
	pub fn by_file_owned(&self, file: &File) -> Option<SStr> {
		if file.is_dir() { Some(Cow::Borrowed(MIME_DIR)) } else { self.by_url_owned(&file.url) }
	}

	#[inline]
	pub fn contains<'a>(&self, url: impl Into<Url<'a>>) -> bool {
		self.0.contains_key(&UrlCov::new(url))
	}

	#[inline]
	pub fn extend(&mut self, iter: impl IntoIterator<Item = (UrlBufCov, String)>) {
		self.0.extend(iter);
	}
}
