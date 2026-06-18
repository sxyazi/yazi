use std::{borrow::Cow, ops::Deref, sync::Arc};

use arc_swap::ArcSwap;
use mlua::{ExternalError, ExternalResult, MetaMethod, UserData, UserDataMethods};
use serde::Deserialize;
use yazi_fs::file::File;
use yazi_shim::{arc_swap::{ArcSwapExt, IntoPointee}, vec::{IndexAtError, VecExt}};

use super::Previewer;
use crate::{mix, plugin::{PreviewerArc, PreviewerMatcher}};

#[derive(Debug, Default, Deserialize)]
pub struct Previewers(ArcSwap<Vec<PreviewerArc>>);

impl Deref for Previewers {
	type Target = ArcSwap<Vec<PreviewerArc>>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl From<Vec<PreviewerArc>> for Previewers {
	fn from(inner: Vec<PreviewerArc>) -> Self { Self(inner.into_pointee()) }
}

impl Previewers {
	pub fn matches(&self, file: &File, mime: &str) -> Option<PreviewerArc> {
		self.matcher(Some(file), Some(mime)).next()
	}

	pub fn matcher<'a, F, M>(&self, file: Option<F>, mime: Option<M>) -> PreviewerMatcher<'a>
	where
		F: Into<Cow<'a, File>>,
		M: Into<Cow<'a, str>>,
	{
		PreviewerMatcher {
			previewers: self.0.load_full(),
			file: file.map(Into::into),
			mime: mime.map(Into::into),
			..Default::default()
		}
	}

	pub fn insert(&self, index: isize, previewer: PreviewerArc) -> Result<(), IndexAtError> {
		self.0.try_rcu(|previewers| {
			let i = previewers.index_at(index)?;
			if i == previewers.len() {
				Ok(mix(Vec::<Previewer>::new(), previewers.iter().cloned(), [previewer.clone()]))
			} else {
				let (before, after) = previewers.split_at(i);
				Ok(mix(
					Vec::<Previewer>::new(),
					before.iter().cloned().chain([previewer.clone()]).chain(after.iter().cloned()),
					Vec::<Previewer>::new(),
				))
			}
		})?;

		Ok(())
	}

	pub fn remove(&self, matcher: PreviewerMatcher) {
		self.0.rcu(|previewers| {
			let mut next = Vec::clone(previewers);
			next.retain(|previewer| !matcher.matches(previewer));
			next
		});
	}

	pub(crate) fn unwrap_unchecked(self) -> Vec<PreviewerArc> {
		Arc::try_unwrap(self.0.into_inner()).expect("unique previewers arc")
	}
}

impl UserData for &'static Previewers {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_method("match", |_, &me, matcher: Option<PreviewerMatcher>| {
			Ok(match matcher {
				Some(matcher) => matcher,
				None => me.into(),
			})
		});

		methods.add_method("insert", |_, &me, (index, previewer): (isize, PreviewerArc)| {
			let index = match index {
				1.. => index - 1,
				0 => return Err("index must be 1-based or negative".into_lua_err()),
				_ => index,
			};

			me.insert(index, previewer.clone()).into_lua_err()?;
			Ok(previewer)
		});

		methods.add_method("remove", |_, &me, matcher: PreviewerMatcher| {
			me.remove(matcher);
			Ok(())
		});

		methods.add_meta_method(MetaMethod::Len, |_, &me, ()| Ok(me.load().len()));
	}
}
