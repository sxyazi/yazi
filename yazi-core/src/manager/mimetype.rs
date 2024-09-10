use std::{collections::HashMap, path::PathBuf};

use yazi_shared::fs::{Url, UrlScheme};

#[derive(Default)]
pub struct Mimetype(HashMap<PathBuf, String>);

impl Mimetype {
	#[inline]
	pub fn get(&self, url: &Url) -> Option<&str> {
		let s = match url.scheme() {
			UrlScheme::Regular => self.0.get(url.as_path()),
			UrlScheme::Search => None,
			UrlScheme::SearchItem => self.0.get(url.as_path()),
			UrlScheme::Archive => None,
		};
		s.map(|s| s.as_str())
	}

	#[inline]
	pub fn get_owned(&self, url: &Url) -> Option<String> { self.get(url).map(|s| s.to_owned()) }

	#[inline]
	pub fn contains(&self, url: &Url) -> bool {
		match url.scheme() {
			UrlScheme::Regular => self.0.contains_key(url.as_path()),
			UrlScheme::Search => false,
			UrlScheme::SearchItem => self.0.contains_key(url.as_path()),
			UrlScheme::Archive => false,
		}
	}

	pub fn extend(&mut self, iter: impl IntoIterator<Item = (Url, String)>) {
		self.0.extend(iter.into_iter().filter_map(|(u, s)| {
			Some((
				match u.scheme() {
					UrlScheme::Regular => u.into_path(),
					UrlScheme::Search => None?,
					UrlScheme::SearchItem => u.into_path(),
					UrlScheme::Archive => None?,
				},
				s,
			))
		}))
	}
}
