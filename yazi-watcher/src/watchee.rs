use std::{hash::Hash, path::Path};

use yazi_shared::url::{AsUrl, Url, UrlCow, UrlLike};

use crate::local::Local;

#[derive(Debug, Eq, Hash, PartialEq)]
pub enum Watchee<'a> {
	Local(UrlCow<'a>, bool),
	Remote(UrlCow<'a>),
}

impl AsUrl for Watchee<'_> {
	fn as_url(&self) -> Url<'_> {
		match self {
			Self::Local(url, _) => url.as_url(),
			Self::Remote(url) => url.as_url(),
		}
	}
}

impl<'a> Watchee<'a> {
	pub(super) fn as_local(&self) -> Option<(&Path, bool)> {
		Some(match self {
			Self::Local(url, alt) => (url.as_local()?, *alt),
			Self::Remote(_) => None?,
		})
	}

	pub(super) fn as_local_mut(&mut self) -> Option<(&Path, &mut bool)> {
		Some(match self {
			Self::Local(url, alt) => (url.as_local()?, alt),
			Self::Remote(_) => None?,
		})
	}

	pub(super) async fn new<U>(url: U) -> Self
	where
		U: Into<UrlCow<'a>>,
	{
		let url = url.into();
		if let Some(path) = url.as_local() {
			let b = Local::soundless(path).await;
			Self::Local(url, b)
		} else {
			Self::Remote(url)
		}
	}

	pub(super) fn to_static(&self) -> Watchee<'static> {
		match self {
			Self::Local(url, alt) => Watchee::Local(url.to_owned().into(), *alt),
			Self::Remote(url) => Watchee::Remote(url.to_owned().into()),
		}
	}
}
