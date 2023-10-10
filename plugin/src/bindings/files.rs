use mlua::{AnyUserData, Function, IntoLua, MetaMethod, UserData, UserDataFields, UserDataMethods, Value};

use super::Url;
use crate::{layout::Style, LUA};

pub struct File(core::files::File);

impl From<&core::files::File> for File {
	fn from(value: &core::files::File) -> Self { Self(value.clone()) }
}

impl UserData for File {
	fn add_fields<'lua, F: UserDataFields<'lua, Self>>(fields: &mut F) {
		fields.add_field_method_get("url", |_, me| Ok(Url::from(me.0.url())));
		fields.add_field_method_get("length", |_, me| Ok(me.0.length()));
		fields.add_field_method_get("link_to", |_, me| Ok(me.0.link_to().map(Url::from)));
		fields.add_field_method_get("is_link", |_, me| Ok(me.0.is_link()));
		fields.add_field_method_get("is_hidden", |_, me| Ok(me.0.is_hidden()));
	}
}

pub struct Files;

impl Files {
	pub(crate) fn init() -> mlua::Result<()> {
		LUA.register_userdata_type::<core::files::Files>(|reg| {
			reg.add_meta_method(MetaMethod::Len, |_, me, ()| Ok(me.len()));

			reg.add_meta_function(MetaMethod::Pairs, |lua, me: AnyUserData| {
				let iter = lua.create_function(|lua, (me, i): (AnyUserData, usize)| {
					let files = me.borrow::<core::files::Files>()?;
					let i = i + 1;
					Ok(if i > files.len() {
						mlua::Variadic::new()
					} else {
						mlua::Variadic::from_iter([i.into_lua(lua)?, File::from(&files[i - 1]).into_lua(lua)?])
					})
				})?;
				Ok((iter, me, 0))
			});

			reg.add_function("slice", |_, (me, skip, take): (AnyUserData, usize, usize)| {
				let files = me.borrow::<core::files::Files>()?;
				Ok(files.iter().skip(skip).take(take).map(File::from).collect::<Vec<_>>())
			});
		})?;

		LUA.register_userdata_type::<core::files::File>(|reg| {
			reg.add_field_method_get("name", |_, me| {
				Ok(me.url().file_name().map(|n| n.to_string_lossy().to_string()))
			});
			reg.add_function("icon", |_, me: AnyUserData| {
				me.named_user_value::<Function>("icon")?.call::<_, String>(())
			});
			reg.add_function("style", |_, me: AnyUserData| {
				me.named_user_value::<Function>("style")?.call::<_, Style>(())
			});
			reg.add_field_function_get("hovered", |_, me| me.named_user_value::<bool>("hovered"));
			reg.add_function("yanked", |_, me: AnyUserData| {
				me.named_user_value::<Function>("yanked")?.call::<_, u8>(me)
			});
			reg.add_function("selected", |_, me: AnyUserData| {
				me.named_user_value::<Function>("selected")?.call::<_, bool>(me)
			});
			reg.add_function("highlights", |_, me: AnyUserData| {
				me.named_user_value::<Function>("highlights")?.call::<_, Value>(())
			});

			reg.add_field_method_get("url", |_, me| Ok(Url::from(me.url())));
			reg.add_field_method_get("length", |_, me| Ok(me.length()));
			reg.add_field_method_get("link_to", |_, me| Ok(me.link_to().map(Url::from)));
			reg.add_field_method_get("is_link", |_, me| Ok(me.is_link()));
			reg.add_field_method_get("is_hidden", |_, me| Ok(me.is_hidden()));

			// Meta
			reg.add_field_method_get("permissions", |_, me| {
				Ok(shared::permissions(me.meta().permissions()))
			});
		})?;

		Ok(())
	}
}
