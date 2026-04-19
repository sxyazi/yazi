use mlua::{IntoLua, MetaMethod, UserData, UserDataMethods};
use yazi_config::YAZI;

use crate::config::FetcherMatcher;

pub struct Fetchers;

impl UserData for Fetchers {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_method("match", |lua, _, matcher: Option<FetcherMatcher>| match matcher {
			Some(matcher) => matcher.into_lua(lua),
			None => FetcherMatcher::new(&YAZI.plugin.fetchers).into_lua(lua),
		});

		methods.add_meta_method(MetaMethod::Len, |_, _, ()| Ok(YAZI.plugin.fetchers.load().len()));
	}
}
