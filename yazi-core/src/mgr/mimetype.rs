use std::{borrow::Cow, collections::HashMap};

use yazi_fs::File;
use yazi_shared::{MIME_DIR, SStr, url::Url};

#[derive(Default)]
pub struct Mimetype(HashMap<Url, String>);

impl Mimetype {
	#[inline]
	pub fn by_url(&self, url: &Url) -> Option<&str> { self.0.get(url).map(|s| s.as_str()) }

	#[inline]
	pub fn by_url_owned(&self, url: &Url) -> Option<SStr> {
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
	pub fn contains(&self, url: &Url) -> bool { self.0.contains_key(url) }

	#[inline]
	pub fn extend(&mut self, iter: impl IntoIterator<Item = (Url, String)>) { self.0.extend(iter) }
}
