use std::{borrow::Cow, ops::Deref, sync::Arc};

use arc_swap::ArcSwap;
use mlua::{MetaMethod, UserData, UserDataMethods};
use serde::Deserialize;
use yazi_fs::file::File;
use yazi_shared::id::Id;
use yazi_shim::arc_swap::IntoPointee;

use crate::plugin::{SpotterArc, SpotterMatcher};

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

	pub fn matcher<'a, F, M>(&self, file: Option<F>, mime: Option<M>) -> SpotterMatcher<'a>
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

		methods.add_meta_method(MetaMethod::Len, |_, me, ()| Ok(me.load().len()));
	}
}
