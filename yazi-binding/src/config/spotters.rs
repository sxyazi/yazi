use mlua::{IntoLua, MetaMethod, UserData, UserDataMethods};
use yazi_config::YAZI;

use crate::config::SpotterMatcher;

pub struct Spotters;

impl UserData for Spotters {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_method("match", |lua, _, matcher: Option<SpotterMatcher>| match matcher {
			Some(matcher) => matcher.into_lua(lua),
			None => SpotterMatcher::new(&YAZI.plugin.spotters).into_lua(lua),
		});

		methods.add_meta_method(MetaMethod::Len, |_, _, ()| Ok(YAZI.plugin.spotters.load().len()));
	}
}
