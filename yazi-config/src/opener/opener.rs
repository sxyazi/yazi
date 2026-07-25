use std::{mem, ops::Deref, sync::Arc};

use arc_swap::ArcSwap;
use hashbrown::HashMap;
use mlua::{ExternalError, FromLua, IntoLua, IntoLuaMulti, LuaString, MetaMethod, UserData, UserDataMethods, Value};
use serde::{Deserialize, Deserializer};
use yazi_shim::{arc_swap::IntoPointee, toml::{DeserializeOverHook, DeserializeOverWith}};

use crate::{open::{OpenRule, OpenRuleArc}, opener::{OpenerRuleArc, OpenerRuleMatcher, OpenerRulesArc, OpenerRulesMatcher}};

#[derive(Debug, Deserialize)]
pub struct Opener(ArcSwap<HashMap<String, OpenerRulesArc>>);

impl Deref for Opener {
	type Target = ArcSwap<HashMap<String, OpenerRulesArc>>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl Opener {
	pub fn all(&self, open: OpenRuleArc) -> impl Iterator<Item = OpenerRuleArc> + use<> {
		let inner = self.0.load_full();
		(0..open.r#use.len())
			.filter_map(move |i| inner.get(&open.r#use[i]).cloned())
			.flat_map(|rules| OpenerRuleMatcher::from(&rules))
	}

	pub fn first(&self, open: &OpenRule) -> Option<OpenerRuleArc> {
		let inner = self.0.load();
		open
			.r#use
			.iter()
			.filter_map(|name| inner.get(name))
			.find_map(|rules| rules.load().first().cloned())
	}

	pub fn block(&self, open: &OpenRule) -> Option<OpenerRuleArc> {
		let inner = self.0.load();
		open
			.r#use
			.iter()
			.filter_map(|name| inner.get(name))
			.find_map(|rules| rules.load().iter().find(|r| r.block).cloned())
	}

	pub fn insert(&self, name: &str, rules: &OpenerRulesArc) {
		self.0.rcu(|inner| {
			let mut next = HashMap::clone(inner);
			next.insert(name.to_owned(), rules.clone());
			next
		});
	}

	pub fn remove(&self, name: &str) {
		self.0.rcu(|inner| {
			let mut next = HashMap::clone(inner);
			next.remove(name);
			next
		});
	}

	pub(crate) fn unwrap_unchecked(self) -> HashMap<String, OpenerRulesArc> {
		Arc::try_unwrap(self.0.into_inner()).expect("unique opener arc")
	}
}

impl DeserializeOverHook for Opener {
	fn deserialize_over_hook(self) -> Result<Self, toml::de::Error> {
		let mut inner = self.unwrap_unchecked();
		for rules in inner.values_mut() {
			*rules = Arc::try_unwrap(mem::take(rules))
				.expect("unique opener value arc")
				.deserialize_over_hook()?
				.into();
		}

		Ok(Self(inner.into_pointee()))
	}
}

impl DeserializeOverWith for Opener {
	fn deserialize_over_with<'de, D: Deserializer<'de>>(self, de: D) -> Result<Self, D::Error> {
		Ok(Self(self.0.deserialize_over_with(de)?))
	}
}

impl UserData for &'static Opener {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_meta_method(MetaMethod::Index, |lua, &me, key: LuaString| {
			let key = key.to_str()?;
			match me.load().get(&*key) {
				Some(rules) => rules.clone().into_lua(lua),
				None => Ok(Value::Nil),
			}
		});

		methods.add_meta_method(MetaMethod::NewIndex, |lua, &me, (key, value): (LuaString, Value)| {
			let key = key.to_str()?;
			match value {
				t @ Value::Table(_) => {
					me.insert(&key, &OpenerRulesArc::from_lua(t, lua)?);
				}
				Value::Nil => {
					me.remove(&key);
				}
				_ => return Err("expected a table or nil".into_lua_err()),
			}
			Ok(())
		});

		methods.add_meta_method(MetaMethod::Pairs, |lua, &me, ()| {
			let mut matcher = OpenerRulesMatcher::from(me);
			let iter = lua.create_function_mut(move |lua, ()| {
				if let Some((name, rules)) = matcher.next() {
					(name, rules).into_lua_multi(lua)
				} else {
					().into_lua_multi(lua)
				}
			})?;

			Ok(iter)
		});
	}
}
