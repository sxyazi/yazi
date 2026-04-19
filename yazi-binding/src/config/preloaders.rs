use mlua::{IntoLua, MetaMethod, UserData, UserDataMethods};
use yazi_config::YAZI;

use crate::config::PreloaderMatcher;

pub struct Preloaders;

impl UserData for Preloaders {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_method("match", |lua, _, matcher: Option<PreloaderMatcher>| match matcher {
			Some(matcher) => matcher.into_lua(lua),
			None => PreloaderMatcher::new(&YAZI.plugin.preloaders).into_lua(lua),
		});

		methods.add_meta_method(MetaMethod::Len, |_, _, ()| Ok(YAZI.plugin.preloaders.load().len()));
	}
}
