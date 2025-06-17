use std::{borrow::Cow, collections::HashMap, path::PathBuf};

use yazi_fs::File;
use yazi_shared::{MIME_DIR, url::{Scheme, Url}};

#[derive(Default)]
pub struct Mimetype(HashMap<PathBuf, String>);

impl Mimetype {
	#[inline]
	pub fn by_url(&self, url: &Url) -> Option<&str> {
		match url.scheme() {
			Scheme::Regular => self.0.get(url.as_path()),
			Scheme::Search => None,
			Scheme::SearchItem => self.0.get(url.as_path()),
			Scheme::Archive => None,
		}
		.map(|s| s.as_str())
	}

	#[inline]
	pub fn by_url_owned(&self, url: &Url) -> Option<Cow<'static, str>> {
		self.by_url(url).map(|s| Cow::Owned(s.to_owned()))
	}

	#[inline]
	pub fn by_file(&self, file: &File) -> Option<&str> {
		if file.is_dir() { Some(MIME_DIR) } else { self.by_url(&file.url) }
	}

	#[inline]
	pub fn by_file_owned(&self, file: &File) -> Option<Cow<'static, str>> {
		if file.is_dir() { Some(Cow::Borrowed(MIME_DIR)) } else { self.by_url_owned(&file.url) }
	}

	#[inline]
	pub fn contains(&self, url: &Url) -> bool {
		match url.scheme() {
			Scheme::Regular => self.0.contains_key(url.as_path()),
			Scheme::Search => false,
			Scheme::SearchItem => self.0.contains_key(url.as_path()),
			Scheme::Archive => false,
		}
	}

	pub fn extend(&mut self, iter: impl IntoIterator<Item = (Url, String)>) {
		self.0.extend(iter.into_iter().filter_map(|(u, s)| {
			Some((
				match u.scheme() {
					Scheme::Regular => u.into_path(),
					Scheme::Search => None?,
					Scheme::SearchItem => u.into_path(),
					Scheme::Archive => None?,
				},
				s,
			))
		}))
	}
}
