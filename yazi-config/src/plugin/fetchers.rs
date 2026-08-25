use std::{ops::Deref, sync::Arc};

use anyhow::{Result, ensure};
use arc_swap::ArcSwap;
use mlua::{ExternalError, ExternalResult, MetaMethod, UserData, UserDataMethods};
use serde::Deserialize;
use yazi_fs::file::File;
use yazi_macro::warn;
use yazi_shim::{arc_swap::{ArcSwapExt, IntoPointee}, vec::VecExt};

use super::{Fetcher, MAX_FETCHERS};
use crate::{mix, plugin::{FetcherArc, FetcherMatcher, fetcher_rev}};

#[derive(Debug, Default, Deserialize)]
pub struct Fetchers(ArcSwap<Vec<FetcherArc>>);

impl Deref for Fetchers {
	type Target = ArcSwap<Vec<FetcherArc>>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl TryFrom<Vec<FetcherArc>> for Fetchers {
	type Error = anyhow::Error;

	fn try_from(inner: Vec<FetcherArc>) -> Result<Self> {
		ensure!(inner.len() <= MAX_FETCHERS as usize, "Fetchers exceed the limit of {MAX_FETCHERS}");

		Ok(Self(Self::reindex(inner).into_pointee()))
	}
}

impl Fetchers {
	pub fn mime(&self, files: Vec<File>) -> impl Iterator<Item = (FetcherArc, Vec<File>)> {
		let fetchers = self.load_full();
		let mut tasks: [Vec<_>; MAX_FETCHERS as usize] = Default::default();

		for file in files {
			let found = FetcherMatcher::new(&fetchers, &file, "").find(|f| f.group == "mime");
			if let Some(fetcher) = found {
				tasks[fetcher.idx as usize].push(file);
			} else {
				warn!("No mime fetcher for {file:?}");
			}
		}

		tasks.into_iter().enumerate().filter_map(move |(i, tasks)| {
			if tasks.is_empty() { None } else { Some((fetchers[i].clone(), tasks)) }
		})
	}

	fn insert(&self, index: isize, fetcher: FetcherArc) -> Result<()> {
		self.0.try_rcu(|fetchers| {
			let i = fetchers.index_at(index)?;
			let next = if i == fetchers.len() {
				mix(Vec::<Fetcher>::new(), fetchers.iter().cloned(), [fetcher.clone()])
			} else {
				let (before, after) = fetchers.split_at(i);
				mix(
					Vec::<Fetcher>::new(),
					before.iter().cloned().chain([fetcher.clone()]).chain(after.iter().cloned()),
					Vec::<Fetcher>::new(),
				)
			};

			ensure!(next.len() <= MAX_FETCHERS as usize, "Fetchers exceed the limit of {MAX_FETCHERS}");
			Ok(Self::reindex(next))
		})?;

		Ok(())
	}

	fn remove(&self, matcher: FetcherMatcher) {
		self.0.rcu(|fetchers| {
			let mut next = Vec::clone(fetchers);
			next.retain(|fetcher| !matcher.matches(fetcher));
			if next.len() == fetchers.len() { next } else { Self::reindex(next) }
		});
	}

	fn reindex(mut fetchers: Vec<FetcherArc>) -> Vec<FetcherArc> {
		let rev = fetcher_rev();
		for (i, fetcher) in fetchers.iter_mut().enumerate() {
			fetcher.idx = i as u8;
			fetcher.rev = rev;
		}
		fetchers
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

		methods.add_method("insert", |_, &me, (index, fetcher): (isize, FetcherArc)| {
			let index = match index {
				1.. => index - 1,
				0 => return Err("index must be 1-based or negative".into_lua_err()),
				_ => index,
			};

			me.insert(index, fetcher.clone()).into_lua_err()?;
			Ok(fetcher)
		});

		methods.add_method("remove", |_, &me, matcher: FetcherMatcher| {
			me.remove(matcher);
			Ok(())
		});

		methods.add_meta_method(MetaMethod::Len, |_, me, ()| Ok(me.load().len()));
	}
}
