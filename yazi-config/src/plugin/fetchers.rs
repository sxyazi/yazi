use std::{borrow::Cow, ops::Deref, sync::Arc};

use arc_swap::ArcSwap;
use serde::Deserialize;
use tracing::warn;
use yazi_fs::File;
use yazi_shim::arc_swap::IntoPointee;

use super::{Fetcher, MAX_FETCHERS};
use crate::plugin::FetcherMatcher;

#[derive(Debug, Default, Deserialize)]
pub struct Fetchers(ArcSwap<Vec<Arc<Fetcher>>>);

impl Deref for Fetchers {
	type Target = ArcSwap<Vec<Arc<Fetcher>>>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl From<Vec<Arc<Fetcher>>> for Fetchers {
	fn from(inner: Vec<Arc<Fetcher>>) -> Self { Self(inner.into_pointee()) }
}

impl From<Arc<Vec<Arc<Fetcher>>>> for Fetchers {
	fn from(inner: Arc<Vec<Arc<Fetcher>>>) -> Self { Self(inner.into()) }
}

impl Fetchers {
	pub fn matches<'a>(&self, file: &'a File, mime: &'a str) -> FetcherMatcher<'a> {
		self.matcher(Some(file), Some(mime))
	}

	pub fn matcher<'a, F, M>(&self, file: Option<F>, mime: Option<M>) -> FetcherMatcher<'a>
	where
		F: Into<Cow<'a, File>>,
		M: Into<Cow<'a, str>>,
	{
		FetcherMatcher {
			fetchers: self.load_full(),
			file: file.map(Into::into),
			mime: mime.map(Into::into),
			..Default::default()
		}
	}

	pub fn mime(&self, files: Vec<File>) -> impl Iterator<Item = (Arc<Fetcher>, Vec<File>)> {
		let mut tasks: [Vec<_>; MAX_FETCHERS as usize] = Default::default();

		for file in files {
			let found = self.matches(&file, "").find(|f| f.group == "mime");
			if let Some(fetcher) = found {
				tasks[fetcher.idx as usize].push(file);
			} else {
				warn!("No mime fetcher for {file:?}");
			}
		}

		let fetchers = self.load();
		tasks.into_iter().enumerate().filter_map(move |(i, tasks)| {
			if tasks.is_empty() { None } else { Some((fetchers[i].clone(), tasks)) }
		})
	}

	pub(crate) fn unwrap_unchecked(self) -> Vec<Arc<Fetcher>> {
		Arc::try_unwrap(self.0.into_inner()).expect("unique fetchers arc")
	}
}
