use std::{borrow::Cow, ops::Deref, sync::Arc};

use arc_swap::ArcSwap;
use mlua::{ExternalError, ExternalResult, MetaMethod, Table, UserData, UserDataMethods};
use serde::Deserialize;
use yazi_fs::file::File;
use yazi_shim::{arc_swap::{ArcSwapExt, IntoPointee}, mlua::DeserializeOverLua, vec::{IndexAtError, VecExt}};

use super::OpenRule;
use crate::{mix, open::{OpenRuleArc, OpenRuleMatcher}};

#[derive(Debug, Default, Deserialize)]
pub struct OpenRules(ArcSwap<Vec<OpenRuleArc>>);

impl Deref for OpenRules {
	type Target = ArcSwap<Vec<OpenRuleArc>>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl From<Vec<OpenRuleArc>> for OpenRules {
	fn from(inner: Vec<OpenRuleArc>) -> Self { Self(inner.into_pointee()) }
}

impl OpenRules {
	pub fn matches(&self, file: &File, mime: &str) -> Option<OpenRuleArc> {
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

	pub fn insert(&self, index: isize, rule: OpenRuleArc) -> Result<(), IndexAtError> {
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

	pub(crate) fn unwrap_unchecked(self) -> Vec<OpenRuleArc> {
		Arc::try_unwrap(self.0.into_inner()).expect("unique open rules arc")
	}
}

impl UserData for &'static OpenRules {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_method("match", |_, &me, matcher: Option<OpenRuleMatcher>| {
			Ok(match matcher {
				Some(matcher) => matcher,
				None => me.into(),
			})
		});

		methods.add_method("insert", |_, &me, (index, rule): (isize, OpenRuleArc)| {
			let index = match index {
				1.. => index - 1,
				0 => return Err("index must be 1-based or negative".into_lua_err()),
				_ => index,
			};

			me.insert(index, rule.clone()).into_lua_err()?;
			Ok(rule)
		});

		methods.add_method("remove", |_, &me, matcher: OpenRuleMatcher| {
			me.remove(matcher);
			Ok(())
		});

		methods.add_method("update", |_, &me, (matcher, table): (OpenRuleMatcher, Table)| {
			me.update(matcher, |rule| rule.deserialize_over_lua(&table))?;
			Ok(())
		});

		methods.add_meta_method(MetaMethod::Len, |_, &me, ()| Ok(me.load().len()));
	}
}
