use std::{borrow::Cow, ops::Deref, sync::Arc};

use arc_swap::ArcSwap;
use mlua::{ExternalError, ExternalResult, MetaMethod, UserData, UserDataMethods};
use serde::Deserialize;
use yazi_fs::file::File;
use yazi_shared::id::Id;
use yazi_shim::{arc_swap::{ArcSwapExt, IntoPointee}, vec::{IndexAtError, VecExt}};

use super::Spotter;
use crate::{mix, plugin::{SpotterArc, SpotterMatcher}};

#[derive(Debug, Default, Deserialize)]
pub struct Spotters(ArcSwap<Vec<SpotterArc>>);

impl Deref for Spotters {
	type Target = ArcSwap<Vec<SpotterArc>>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl From<Vec<SpotterArc>> for Spotters {
	fn from(inner: Vec<SpotterArc>) -> Self { Self(inner.into_pointee()) }
}

impl Spotters {
	pub fn matches(&self, file: &File, mime: &str) -> Option<SpotterArc> {
		self.matcher(Some(file), Some(mime)).next()
	}

	fn matcher<'a, F, M>(&self, file: Option<F>, mime: Option<M>) -> SpotterMatcher<'a>
	where
		F: Into<Cow<'a, File>>,
		M: Into<Cow<'a, str>>,
	{
		SpotterMatcher {
			spotters: self.load_full(),
			id: Id::ZERO,
			file: file.map(Into::into),
			mime: mime.map(Into::into),
			..Default::default()
		}
	}

	fn insert(&self, index: isize, spotter: SpotterArc) -> Result<(), IndexAtError> {
		self.0.try_rcu(|spotters| {
			let i = spotters.index_at(index)?;
			if i == spotters.len() {
				Ok(mix(Vec::<Spotter>::new(), spotters.iter().cloned(), [spotter.clone()]))
			} else {
				let (before, after) = spotters.split_at(i);
				Ok(mix(
					Vec::<Spotter>::new(),
					before.iter().cloned().chain([spotter.clone()]).chain(after.iter().cloned()),
					Vec::<Spotter>::new(),
				))
			}
		})?;

		Ok(())
	}

	fn remove(&self, matcher: SpotterMatcher) {
		self.0.rcu(|spotters| {
			let mut next = Vec::clone(spotters);
			next.retain(|spotter| !matcher.matches(spotter));
			next
		});
	}

	pub(crate) fn unwrap_unchecked(self) -> Vec<SpotterArc> {
		Arc::try_unwrap(self.0.into_inner()).expect("unique spotters arc")
	}
}

impl UserData for &'static Spotters {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_method("match", |_, &me, matcher: Option<SpotterMatcher>| {
			Ok(match matcher {
				Some(matcher) => matcher,
				None => me.into(),
			})
		});

		methods.add_method("insert", |_, &me, (index, spotter): (isize, SpotterArc)| {
			let index = match index {
				1.. => index - 1,
				0 => return Err("index must be 1-based or negative".into_lua_err()),
				_ => index,
			};

			me.insert(index, spotter.clone()).into_lua_err()?;
			Ok(spotter)
		});

		methods.add_method("remove", |_, &me, matcher: SpotterMatcher| {
			me.remove(matcher);
			Ok(())
		});

		methods.add_meta_method(MetaMethod::Len, |_, me, ()| Ok(me.load().len()));
	}
}
