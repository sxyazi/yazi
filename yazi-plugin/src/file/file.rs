use mlua::{AnyUserData, Lua, Table, UserDataFields, UserDataMethods, UserDataRef, UserDataRegistry};
use yazi_config::THEME;
use yazi_shared::fs::Loc;

use crate::{bindings::{Cast, Icon}, cha::Cha, url::Url};

pub type FileRef<'lua> = UserDataRef<'lua, yazi_shared::fs::File>;

pub struct File;

impl File {
	#[inline]
	pub fn register(lua: &Lua) -> mlua::Result<()> {
		lua.register_userdata_type::<yazi_shared::fs::File>(Self::register_with)
	}

	pub fn register_with<T>(reg: &mut UserDataRegistry<T>)
	where
		T: AsRef<yazi_shared::fs::File>,
	{
		reg.add_field_method_get("cha", |lua, me| Cha::cast(lua, me.as_ref().cha));
		reg.add_field_method_get("url", |lua, me| Url::cast(lua, me.as_ref().url_owned()));
		reg.add_field_method_get("link_to", |lua, me| {
			me.as_ref().link_to.clone().map(|u| Url::cast(lua, u)).transpose()
		});

		// Extension
		reg.add_field_method_get("name", |lua, me| {
			me.as_ref().url().file_name().map(|n| lua.create_string(n.as_encoded_bytes())).transpose()
		});

		reg.add_method("icon", |lua, me, ()| {
			use yazi_shared::theme::IconCache;

			let me = me.as_ref();
			match me.icon.get() {
				IconCache::Missing => {
					let matched = THEME.icons.matches(me);
					me.icon.set(matched.map_or(IconCache::Undefined, IconCache::Icon));
					matched.map(|i| Icon::cast(lua, i)).transpose()
				}
				IconCache::Undefined => Ok(None),
				IconCache::Icon(cached) => Some(Icon::cast(lua, cached)).transpose(),
			}
		});
	}

	pub fn install(lua: &Lua) -> mlua::Result<()> {
		lua.globals().raw_set(
			"File",
			lua.create_function(|lua, t: Table| {
				Self::cast(lua, yazi_shared::fs::File {
					loc: Loc::from(t.raw_get::<_, AnyUserData>("url")?.take()?),
					cha: t.raw_get::<_, AnyUserData>("cha")?.take()?,
					..Default::default()
				})
			})?,
		)
	}
}

impl<T: Into<yazi_shared::fs::File>> Cast<T> for File {
	fn cast(lua: &Lua, data: T) -> mlua::Result<AnyUserData> { lua.create_any_userdata(data.into()) }
}
