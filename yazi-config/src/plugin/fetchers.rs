use std::{borrow::Cow, ops::Deref, sync::Arc};

use arc_swap::ArcSwap;
use mlua::{MetaMethod, UserData, UserDataMethods};
use serde::Deserialize;
use tracing::warn;
use yazi_fs::file::File;
use yazi_shim::arc_swap::IntoPointee;

use super::MAX_FETCHERS;
use crate::plugin::{FetcherArc, FetcherMatcher};

#[derive(Debug, Default, Deserialize)]
pub struct Fetchers(ArcSwap<Vec<FetcherArc>>);

impl Deref for Fetchers {
	type Target = ArcSwap<Vec<FetcherArc>>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl From<Vec<FetcherArc>> for Fetchers {
	fn from(inner: Vec<FetcherArc>) -> Self { Self(inner.into_pointee()) }
}

impl From<Arc<Vec<FetcherArc>>> for Fetchers {
	fn from(inner: Arc<Vec<FetcherArc>>) -> Self { Self(inner.into()) }
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

	pub fn mime(&self, files: Vec<File>) -> impl Iterator<Item = (FetcherArc, Vec<File>)> {
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

	pub(crate) fn unwrap_unchecked(self) -> Vec<FetcherArc> {
		Arc::try_unwrap(self.0.into_inner()).expect("unique fetchers arc")
	}
}

impl UserData for &'static Fetchers {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_method("match", |_, &me, matcher: Option<FetcherMatcher>| {
			Ok(match matcher {
				Some(matcher) => matcher,
				None => me.into(),
			})
		});

		methods.add_meta_method(MetaMethod::Len, |_, me, ()| Ok(me.load().len()));
	}
}
