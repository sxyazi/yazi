use config::THEME;
use mlua::{AnyUserData, IntoLua, MetaMethod, UserData, UserDataFields, UserDataMethods, UserDataRef};

use super::{Range, Url};
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
			reg.add_field_method_get("url", |_, me| Ok(Url::from(me.url())));
			reg.add_field_method_get("length", |_, me| Ok(me.length()));
			reg.add_field_method_get("link_to", |_, me| Ok(me.link_to().map(Url::from)));
			reg.add_field_method_get("is_link", |_, me| Ok(me.is_link()));
			reg.add_field_method_get("is_hidden", |_, me| Ok(me.is_hidden()));

			reg.add_field_method_get("name", |_, me| {
				Ok(me.url().file_name().map(|n| n.to_string_lossy().to_string()))
			});
			reg.add_field_method_get("permissions", |_, me| {
				Ok(shared::permissions(me.meta().permissions()))
			});

			reg.add_function("prefix", |_, me: AnyUserData| {
				let folder = me.named_user_value::<UserDataRef<core::manager::Folder>>("folder")?;
				if !folder.cwd.is_search() {
					return Ok(None);
				}

				let file = me.borrow::<core::files::File>()?;
				let mut p = file.url().strip_prefix(&folder.cwd).unwrap_or(file.url()).components();
				p.next_back();
				Ok(Some(p.as_path().to_string_lossy().to_string()))
			});

			reg.add_method("icon", |_, me, ()| {
				Ok(
					THEME
						.icons
						.iter()
						.find(|&x| x.name.match_path(me.url(), Some(me.is_dir())))
						.map(|x| x.display.to_string()),
				)
			});

			reg.add_function("style", |_, me: AnyUserData| {
				let manager = me.named_user_value::<UserDataRef<core::manager::Manager>>("manager")?;
				let file = me.borrow::<core::files::File>()?;
				let mime = manager.mimetype.get(file.url());
				Ok(
					THEME
						.filetypes
						.iter()
						.find(|&x| x.matches(file.url(), mime, file.is_dir()))
						.map(|x| Style::from(x.style)),
				)
			});

			reg.add_function("is_hovered", |_, me: AnyUserData| {
				let folder = me.named_user_value::<UserDataRef<core::manager::Folder>>("folder")?;
				let file = me.borrow::<core::files::File>()?;
				Ok(matches!(&folder.hovered, Some(f) if f.url() == file.url()))
			});

			reg.add_function("is_yanked", |_, me: AnyUserData| {
				let manager = me.named_user_value::<UserDataRef<core::manager::Manager>>("manager")?;
				let file = me.borrow::<core::files::File>()?;
				Ok(if !manager.yanked().1.contains(file.url()) {
					0u8
				} else if manager.yanked().0 {
					2u8
				} else {
					1u8
				})
			});

			reg.add_function("is_selected", |_, me: AnyUserData| {
				let manager = me.named_user_value::<UserDataRef<core::manager::Manager>>("manager")?;
				let folder = me.named_user_value::<UserDataRef<core::manager::Folder>>("folder")?;
				let file = me.borrow::<core::files::File>()?;

				let selected = folder.files.is_selected(file.url());
				Ok(if !manager.active().mode.is_visual() {
					selected
				} else {
					let idx: usize = me.named_user_value("idx")?;
					manager.active().mode.pending(folder.offset() + idx, selected)
				})
			});

			reg.add_function("found", |lua, me: AnyUserData| {
				let manager = me.named_user_value::<UserDataRef<core::manager::Manager>>("manager")?;
				let Some(finder) = manager.active().finder() else {
					return Ok(None);
				};

				let file = me.borrow::<core::files::File>()?;
				if let Some(idx) = finder.matched_idx(file.url()) {
					return Some(
						lua.create_sequence_from([idx.into_lua(lua)?, finder.matched().len().into_lua(lua)?]),
					)
					.transpose();
				}
				Ok(None)
			});

			reg.add_function("highlights", |_, me: AnyUserData| {
				let manager = me.named_user_value::<UserDataRef<core::manager::Manager>>("manager")?;
				let Some(finder) = manager.active().finder() else {
					return Ok(None);
				};

				let file = me.borrow::<core::files::File>()?;
				let Some(h) = file.name().and_then(|n| finder.highlighted(n)) else {
					return Ok(None);
				};

				Ok(Some(h.into_iter().map(Range::from).collect::<Vec<_>>()))
			});
		})?;

		Ok(())
	}
}
