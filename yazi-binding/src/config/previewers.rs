use mlua::{ExternalError, ExternalResult, FromLua, IntoLua, Lua, MetaMethod, Table, UserData, UserDataMethods, Value};
use yazi_config::YAZI;

use crate::{FileRef, Id, Iter, config::Previewer};

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

// --- Matcher
pub struct PreviewerMatcher(yazi_config::plugin::PreviewerMatcher<'static>);

impl PreviewerMatcher {
	fn new(inner: impl Into<yazi_config::plugin::PreviewerMatcher<'static>>) -> Self {
		Self(inner.into())
	}
}

impl TryFrom<Table> for PreviewerMatcher {
	type Error = mlua::Error;

	fn try_from(value: Table) -> Result<Self, Self::Error> {
		let id: Id = value.raw_get("id").unwrap_or_default();
		let file: Option<FileRef> = value.raw_get("file")?;
		let mime: Option<String> = value.raw_get("mime")?;

		Ok(Self(yazi_config::plugin::PreviewerMatcher {
			previewers: YAZI.plugin.previewers.load_full(),
			id: id.0,
			file: file.map(|f| f.inner.clone().into()),
			mime: mime.map(Into::into),
			..Default::default()
		}))
	}
}

impl FromLua for PreviewerMatcher {
	fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
		match value {
			Value::Table(t) => t.try_into(),
			_ => return Err("expected a table of PreviewerMatcher".into_lua_err()),
		}
	}
}

impl IntoLua for PreviewerMatcher {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		Iter::new(self.0.into_iter().map(Previewer::new), None).into_lua(lua)
	}
}
