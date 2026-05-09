use mlua::{ExternalError, ExternalResult, IntoLua, MetaMethod, Table, UserData, UserDataMethods};
use yazi_config::YAZI;
use yazi_shim::mlua::DeserializeOverLua;

use crate::config::{OpenRule, OpenRuleMatcher};

pub struct OpenRules;

impl UserData for OpenRules {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_method("match", |lua, _, matcher: Option<OpenRuleMatcher>| match matcher {
			Some(matcher) => matcher.into_lua(lua),
			None => OpenRuleMatcher::new(&*YAZI.open).into_lua(lua),
		});

		methods.add_method("insert", |_, _, (index, rule): (isize, OpenRule)| {
			let index = match index {
				1.. => index - 1,
				0 => return Err("index must be 1-based or negative".into_lua_err()),
				_ => index,
			};

			YAZI.open.insert(index, rule.clone()).into_lua_err()?;
			Ok(rule)
		});

		methods.add_method("remove", |_, _, matcher: OpenRuleMatcher| {
			YAZI.open.remove(matcher.0);
			Ok(())
		});

		methods.add_method("update", |_, _, (matcher, table): (OpenRuleMatcher, Table)| {
			YAZI.open.update(matcher.0, |rule| rule.deserialize_over_lua(&table))?;
			Ok(())
		});

		methods.add_meta_method(MetaMethod::Len, |_, _, ()| Ok(YAZI.open.load().len()));
	}
}
