use mlua::{ExternalError, FromLua, IntoLua, IntoLuaMulti, MetaMethod, UserData, UserDataMethods, Value};
use yazi_config::{YAZI, opener::OpenerRulesMatcher};

use crate::config::OpenerRules;

pub struct Opener;

impl UserData for Opener {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_meta_method(MetaMethod::Index, |lua, _, key: mlua::String| {
			let key = key.to_str()?;
			match YAZI.opener.load().get(&*key) {
				Some(rules) => OpenerRules::new(rules.clone()).into_lua(lua),
				None => Ok(Value::Nil),
			}
		});

		methods.add_meta_method(MetaMethod::NewIndex, |lua, _, (key, value): (mlua::String, Value)| {
			let key = key.to_str()?;
			match value {
				t @ Value::Table(_) => {
					YAZI.opener.insert(&key, &*OpenerRules::from_lua(t, lua)?);
				}
				Value::Nil => {
					YAZI.opener.remove(&key);
				}
				_ => return Err("expected a table or nil".into_lua_err()),
			}
			Ok(())
		});

		methods.add_meta_method(MetaMethod::Pairs, |lua, _, ()| {
			let mut matcher = OpenerRulesMatcher::from(&YAZI.opener);
			let iter = lua.create_function_mut(move |lua, ()| {
				if let Some((name, rules)) = matcher.next() {
					(name, OpenerRules::new(rules)).into_lua_multi(lua)
				} else {
					().into_lua_multi(lua)
				}
			})?;

			Ok(iter)
		});
	}
}
