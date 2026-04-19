use std::{borrow::Cow, ops::Deref, sync::Arc};

use arc_swap::ArcSwap;
use serde::Deserialize;
use yazi_fs::File;
use yazi_shared::Id;
use yazi_shim::{arc_swap::{ArcSwapExt, IntoPointee}, vec::{IndexAtError, VecExt}};

use super::OpenRule;
use crate::{Selectable, mix};

#[derive(Debug, Default, Deserialize)]
pub struct OpenRules(ArcSwap<Vec<Arc<OpenRule>>>);

impl Deref for OpenRules {
	type Target = ArcSwap<Vec<Arc<OpenRule>>>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl From<Vec<Arc<OpenRule>>> for OpenRules {
	fn from(inner: Vec<Arc<OpenRule>>) -> Self { Self(inner.into_pointee()) }
}

impl OpenRules {
	pub fn matches(&self, file: &File, mime: &str) -> Option<Arc<OpenRule>> {
		self.matcher(Some(file), Some(mime)).next()
	}

	pub fn matcher<'a, F, M>(&self, file: Option<F>, mime: Option<M>) -> OpenRuleMatcher<'a>
	where
		F: Into<Cow<'a, File>>,
		M: Into<Cow<'a, str>>,
	{
		OpenRuleMatcher {
			rules: self.0.load_full(),
			file: file.map(Into::into),
			mime: mime.map(Into::into),
			..Default::default()
		}
	}

	pub fn insert(&self, index: isize, rule: Arc<OpenRule>) -> Result<(), IndexAtError> {
		self.0.try_rcu(|rules| {
			let i = rules.index_at(index)?;
			Ok(if i == rules.len() {
				mix(Vec::<OpenRule>::new(), rules.iter().cloned(), [rule.clone()])
			} else {
				let (before, after) = rules.split_at(i);
				mix(
					Vec::<OpenRule>::new(),
					before.iter().cloned().chain([rule.clone()]).chain(after.iter().cloned()),
					Vec::<OpenRule>::new(),
				)
			})
		})?;

		Ok(())
	}

	pub fn remove(&self, matcher: OpenRuleMatcher) {
		self.0.rcu(|rules| {
			let mut next = Vec::clone(rules);
			next.retain(|rule| !matcher.matches(rule));
			next
		});
	}

	pub fn update<E>(
		&self,
		matcher: OpenRuleMatcher,
		f: impl Fn(OpenRule) -> Result<OpenRule, E>,
	) -> Result<(), E> {
		self.0.try_rcu(|rules| {
			let mut next = Vec::clone(rules);
			for rule in &mut next {
				if matcher.matches(rule) {
					*rule = f(OpenRule::clone(rule))?.into();
				}
			}
			Ok(Arc::new(next))
		})?;

		Ok(())
	}

	pub(crate) fn unwrap_unchecked(self) -> Vec<Arc<OpenRule>> {
		Arc::try_unwrap(self.0.into_inner()).expect("unique open rules arc")
	}
}

// --- Matcher
#[derive(Default)]
pub struct OpenRuleMatcher<'a> {
	pub rules:  Arc<Vec<Arc<OpenRule>>>,
	pub id:     Id,
	pub file:   Option<Cow<'a, File>>,
	pub mime:   Option<Cow<'a, str>>,
	pub all:    bool,
	pub offset: usize,
}

impl From<&OpenRules> for OpenRuleMatcher<'_> {
	fn from(rules: &OpenRules) -> Self {
		Self { rules: rules.load_full(), all: true, ..Default::default() }
	}
}

impl OpenRuleMatcher<'_> {
	pub fn matches(&self, rule: &OpenRule) -> bool {
		if self.all {
			true
		} else if self.id != Id::ZERO {
			rule.id == self.id
		} else {
			rule.match_with(self.file.as_deref(), self.mime.as_deref())
		}
	}
}

impl Iterator for OpenRuleMatcher<'_> {
	type Item = Arc<OpenRule>;

	fn next(&mut self) -> Option<Self::Item> {
		while let Some(rule) = self.rules.get(self.offset) {
			self.offset += 1;
			if self.matches(rule) {
				return Some(rule.clone());
			}
		}
		None
	}
}
