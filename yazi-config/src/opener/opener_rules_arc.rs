use std::{ops::{Deref, DerefMut}, sync::Arc};

use mlua::{ExternalError, ExternalResult, FromLua, Lua, LuaSerdeExt, MetaMethod, UserData, UserDataMethods, Value};
use serde::Deserialize;
use yazi_shim::toml::DeserializeOverHook;

use crate::opener::{OpenerRuleArc, OpenerRuleMatcher, OpenerRules};

#[derive(Clone, Debug, Deserialize)]
pub struct OpenerRulesArc(Arc<OpenerRules>);

impl Deref for OpenerRulesArc {
	type Target = Arc<OpenerRules>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl DerefMut for OpenerRulesArc {
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl From<OpenerRules> for OpenerRulesArc {
	fn from(value: OpenerRules) -> Self { Self(value.into()) }
}

impl FromLua for OpenerRulesArc {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		let inner: OpenerRules = lua.from_value(value)?;

		Ok(inner.deserialize_over_hook().into_lua_err()?.into())
	}
}

impl UserData for OpenerRulesArc {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_method("match", |_, me, matcher: Option<OpenerRuleMatcher>| {
			Ok(match matcher {
				Some(matcher) => matcher,
				None => me.into(),
			})
		});

		methods.add_method("insert", |_, me, (index, rule): (isize, OpenerRuleArc)| {
			let index = match index {
				1.. => index - 1,
				0 => return Err("index must be 1-based or negative".into_lua_err()),
				_ => index,
			};

			me.insert(index, rule.clone()).into_lua_err()?;
			Ok(rule)
		});

		methods.add_method("remove", |_, me, matcher: OpenerRuleMatcher| {
			me.remove(matcher);
			Ok(())
		});

		methods.add_meta_method(MetaMethod::Len, |_, me, ()| Ok(me.load().len()));
	}
}
