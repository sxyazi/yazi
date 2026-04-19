use mlua::{ExternalError, ExternalResult, IntoLua, MetaMethod, UserData, UserDataMethods};
use yazi_config::YAZI;

use crate::config::{Previewer, PreviewerMatcher};

pub struct Previewers;

impl UserData for Previewers {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_method("match", |lua, _, matcher: Option<PreviewerMatcher>| match matcher {
			Some(matcher) => matcher.into_lua(lua),
			None => PreviewerMatcher::new(&YAZI.plugin.previewers).into_lua(lua),
		});

		methods.add_method("insert", |_, _, (index, previewer): (isize, Previewer)| {
			let index = match index {
				1.. => index - 1,
				0 => return Err("index must be 1-based or negative".into_lua_err()),
				_ => index,
			};

			YAZI.plugin.previewers.insert(index, previewer.clone().into()).into_lua_err()?;
			Ok(previewer)
		});

		methods.add_method("remove", |_, _, matcher: PreviewerMatcher| {
			YAZI.plugin.previewers.remove(matcher.0);
			Ok(())
		});

		methods.add_meta_method(MetaMethod::Len, |_, _, ()| Ok(YAZI.plugin.previewers.load().len()));
	}
}
