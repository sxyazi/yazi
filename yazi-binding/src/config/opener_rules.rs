use std::{ops::Deref, sync::Arc};

use mlua::{ExternalError, ExternalResult, FromLua, IntoLua, Lua, LuaSerdeExt, MetaMethod, UserData, UserDataMethods, Value};
use yazi_shim::toml::DeserializeOverHook;

use crate::config::{OpenerRule, OpenerRuleMatcher};

pub struct OpenerRules {
	inner: Arc<yazi_config::opener::OpenerRules>,
}

impl Deref for OpenerRules {
	type Target = Arc<yazi_config::opener::OpenerRules>;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl OpenerRules {
	pub fn new(inner: impl Into<Arc<yazi_config::opener::OpenerRules>>) -> Self {
		Self { inner: inner.into() }
	}
}

impl FromLua for OpenerRules {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		let inner: yazi_config::opener::OpenerRules = lua.from_value(value)?;

		Ok(Self::new(inner.deserialize_over_hook().into_lua_err()?))
	}
}

impl UserData for OpenerRules {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_method("match", |lua, me, matcher: Option<OpenerRuleMatcher>| match matcher {
			Some(matcher) => matcher.into_lua(lua),
			None => OpenerRuleMatcher::new(&*me.inner).into_lua(lua),
		});

		methods.add_method("insert", |_, me, (index, rule): (isize, OpenerRule)| {
			let index = match index {
				1.. => index - 1,
				0 => return Err("index must be 1-based or negative".into_lua_err()),
				_ => index,
			};

			me.inner.insert(index, rule.clone()).into_lua_err()?;
			Ok(rule)
		});

		methods.add_method("remove", |_, me, matcher: OpenerRuleMatcher| {
			me.inner.remove(matcher.0);
			Ok(())
		});

		methods.add_meta_method(MetaMethod::Len, |_, me, ()| Ok(me.inner.load().len()));
	}
}
