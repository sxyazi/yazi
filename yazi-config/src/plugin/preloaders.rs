use std::{borrow::Cow, ops::Deref, sync::Arc};

use anyhow::{Result, ensure};
use arc_swap::ArcSwap;
use mlua::{ExternalError, ExternalResult, MetaMethod, UserData, UserDataMethods};
use serde::Deserialize;
use yazi_fs::file::File;
use yazi_shim::{arc_swap::{ArcSwapExt, IntoPointee}, vec::VecExt};

use super::{MAX_PRELOADERS, Preloader};
use crate::{mix, plugin::{PreloaderArc, PreloaderMatcher, preloader_rev}};

#[derive(Debug, Default, Deserialize)]
pub struct Preloaders(ArcSwap<Vec<PreloaderArc>>);

impl Deref for Preloaders {
	type Target = ArcSwap<Vec<PreloaderArc>>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl TryFrom<Vec<PreloaderArc>> for Preloaders {
	type Error = anyhow::Error;

	fn try_from(inner: Vec<PreloaderArc>) -> Result<Self> {
		ensure!(
			inner.len() <= MAX_PRELOADERS as usize,
			"Preloaders exceed the limit of {MAX_PRELOADERS}"
		);

		Ok(Self(Self::reindex(inner).into_pointee()))
	}
}

impl Preloaders {
	pub fn matches<'a>(&self, file: &'a File, mime: &'a str) -> PreloaderMatcher<'a> {
		self.matcher(Some(file), Some(mime))
	}

	pub fn matcher<'a, F, M>(&self, file: Option<F>, mime: Option<M>) -> PreloaderMatcher<'a>
	where
		F: Into<Cow<'a, File>>,
		M: Into<Cow<'a, str>>,
	{
		PreloaderMatcher {
			preloaders: self.load_full(),
			file: file.map(Into::into),
			mime: mime.map(Into::into),
			..Default::default()
		}
	}

	pub fn insert(&self, index: isize, preloader: PreloaderArc) -> Result<()> {
		self.0.try_rcu(|preloaders| {
			let i = preloaders.index_at(index)?;
			let next = if i == preloaders.len() {
				mix(Vec::<Preloader>::new(), preloaders.iter().cloned(), [preloader.clone()])
			} else {
				let (before, after) = preloaders.split_at(i);
				mix(
					Vec::<Preloader>::new(),
					before.iter().cloned().chain([preloader.clone()]).chain(after.iter().cloned()),
					Vec::<Preloader>::new(),
				)
			};

			ensure!(
				next.len() <= MAX_PRELOADERS as usize,
				"Preloaders exceed the limit of {MAX_PRELOADERS}"
			);
			Ok(Self::reindex(next))
		})?;

		Ok(())
	}

	pub fn remove(&self, matcher: PreloaderMatcher) {
		self.0.rcu(|preloaders| {
			let mut next = Vec::clone(preloaders);
			next.retain(|preloader| !matcher.matches(preloader));
			if next.len() == preloaders.len() { next } else { Self::reindex(next) }
		});
	}

	fn reindex(mut preloaders: Vec<PreloaderArc>) -> Vec<PreloaderArc> {
		let rev = preloader_rev();
		for (i, preloader) in preloaders.iter_mut().enumerate() {
			preloader.idx = i as u8;
			preloader.rev = rev;
		}
		preloaders
	}

	pub(crate) fn unwrap_unchecked(self) -> Vec<PreloaderArc> {
		Arc::try_unwrap(self.0.into_inner()).expect("unique preloaders arc")
	}
}

impl UserData for &'static Preloaders {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_method("match", |_, &me, matcher: Option<PreloaderMatcher>| {
			Ok(match matcher {
				Some(matcher) => matcher,
				None => me.into(),
			})
		});

		methods.add_method("insert", |_, &me, (index, preloader): (isize, PreloaderArc)| {
			let index = match index {
				1.. => index - 1,
				0 => return Err("index must be 1-based or negative".into_lua_err()),
				_ => index,
			};

			me.insert(index, preloader.clone()).into_lua_err()?;
			Ok(preloader)
		});

		methods.add_method("remove", |_, &me, matcher: PreloaderMatcher| {
			me.remove(matcher);
			Ok(())
		});

		methods.add_meta_method(MetaMethod::Len, |_, me, ()| Ok(me.load().len()));
	}
}
