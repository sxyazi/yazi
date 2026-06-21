use std::{borrow::Cow, ops::Deref, sync::Arc};

use arc_swap::ArcSwap;
use mlua::{MetaMethod, UserData, UserDataMethods};
use serde::Deserialize;
use yazi_fs::file::File;
use yazi_shim::arc_swap::IntoPointee;

use crate::plugin::{PreloaderArc, PreloaderMatcher};

#[derive(Debug, Default, Deserialize)]
pub struct Preloaders(ArcSwap<Vec<PreloaderArc>>);

impl Deref for Preloaders {
	type Target = ArcSwap<Vec<PreloaderArc>>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl From<Vec<PreloaderArc>> for Preloaders {
	fn from(inner: Vec<PreloaderArc>) -> Self { Self(inner.into_pointee()) }
}

impl From<Arc<Vec<PreloaderArc>>> for Preloaders {
	fn from(inner: Arc<Vec<PreloaderArc>>) -> Self { Self(inner.into()) }
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

		methods.add_meta_method(MetaMethod::Len, |_, me, ()| Ok(me.load().len()));
	}
}
