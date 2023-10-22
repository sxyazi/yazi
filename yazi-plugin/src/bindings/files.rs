use std::time::UNIX_EPOCH;

use mlua::{AnyUserData, IntoLua, MetaMethod, UserData, UserDataFields, UserDataMethods, UserDataRef};
use yazi_config::THEME;

use super::shared::{Range, Url};
use crate::{layout::Style, LUA};

pub struct File(yazi_core::files::File);

impl From<&yazi_core::files::File> for File {
	fn from(value: &yazi_core::files::File) -> Self { Self(value.clone()) }
}

impl UserData for File {
	fn add_fields<'lua, F: UserDataFields<'lua, Self>>(fields: &mut F) {
		fields.add_field_method_get("url", |_, me| Ok(Url::from(&me.0.url)));
		fields.add_field_method_get("link_to", |_, me| Ok(me.0.link_to().map(Url::from)));
		fields.add_field_method_get("is_link", |_, me| Ok(me.0.is_link));
		fields.add_field_method_get("is_hidden", |_, me| Ok(me.0.is_hidden));
	}
}

pub struct Files;

impl Files {
	pub(crate) fn init() -> mlua::Result<()> {
		LUA.register_userdata_type::<yazi_core::files::Files>(|reg| {
			reg.add_meta_method(MetaMethod::Len, |_, me, ()| Ok(me.len()));

			reg.add_meta_function(MetaMethod::Pairs, |lua, me: AnyUserData| {
				let iter = lua.create_function(|lua, (me, i): (AnyUserData, usize)| {
					let files = me.borrow::<yazi_core::files::Files>()?;
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
				let files = me.borrow::<yazi_core::files::Files>()?;
				Ok(files.iter().skip(skip).take(take).map(File::from).collect::<Vec<_>>())
			});
		})?;

		LUA.register_userdata_type::<yazi_core::files::File>(|reg| {
			reg.add_field_method_get("url", |_, me| Ok(Url::from(&me.url)));
			reg.add_field_method_get("link_to", |_, me| Ok(me.link_to().map(Url::from)));
			reg.add_field_method_get("is_link", |_, me| Ok(me.is_link));
			reg.add_field_method_get("is_hidden", |_, me| Ok(me.is_hidden));

			// Metadata
			reg.add_field_method_get("is_file", |_, me| Ok(me.is_file()));
			reg.add_field_method_get("is_dir", |_, me| Ok(me.is_dir()));
			reg.add_field_method_get("is_symlink", |_, me| Ok(me.meta.is_symlink()));
			#[cfg(unix)]
			{
				use std::os::unix::prelude::FileTypeExt;
				reg.add_field_method_get("is_block_device", |_, me| {
					Ok(me.meta.file_type().is_block_device())
				});
				reg
					.add_field_method_get("is_char_device", |_, me| Ok(me.meta.file_type().is_char_device()));
				reg.add_field_method_get("is_fifo", |_, me| Ok(me.meta.file_type().is_fifo()));
				reg.add_field_method_get("is_socket", |_, me| Ok(me.meta.file_type().is_socket()));
			}
			reg.add_field_method_get("length", |_, me| Ok(me.meta.len()));
			reg.add_field_method_get("created", |_, me| {
				Ok(me.meta.created()?.duration_since(UNIX_EPOCH).map(|d| d.as_secs_f64()).ok())
			});
			reg.add_field_method_get("modified", |_, me| {
				Ok(me.meta.modified()?.duration_since(UNIX_EPOCH).map(|d| d.as_secs_f64()).ok())
			});
			reg.add_field_method_get("accessed", |_, me| {
				Ok(me.meta.accessed()?.duration_since(UNIX_EPOCH).map(|d| d.as_secs_f64()).ok())
			});
			reg
				.add_method("permissions", |_, me, ()| Ok(yazi_shared::permissions(me.meta.permissions())));

			// Extension
			reg.add_field_method_get("name", |_, me| {
				Ok(me.url.file_name().map(|n| n.to_string_lossy().to_string()))
			});
			reg.add_function("size", |_, me: AnyUserData| {
				let file = me.borrow::<yazi_core::files::File>()?;
				if !file.is_dir() {
					return Ok(Some(file.meta.len()));
				}

				let folder = me.named_user_value::<UserDataRef<yazi_core::tab::Folder>>("folder")?;
				Ok(folder.files.sizes.get(&file.url).copied())
			});
			reg.add_function("mime", |_, me: AnyUserData| {
				let manager = me.named_user_value::<UserDataRef<yazi_core::manager::Manager>>("manager")?;
				let file = me.borrow::<yazi_core::files::File>()?;
				Ok(manager.mimetype.get(&file.url).cloned())
			});
			reg.add_function("prefix", |_, me: AnyUserData| {
				let folder = me.named_user_value::<UserDataRef<yazi_core::tab::Folder>>("folder")?;
				if !folder.cwd.is_search() {
					return Ok(None);
				}

				let file = me.borrow::<yazi_core::files::File>()?;
				let mut p = file.url.strip_prefix(&folder.cwd).unwrap_or(&file.url).components();
				p.next_back();
				Ok(Some(p.as_path().to_string_lossy().to_string()))
			});
			reg.add_method("icon", |_, me, ()| {
				Ok(
					THEME
						.icons
						.iter()
						.find(|&x| x.name.match_path(&me.url, Some(me.is_dir())))
						.map(|x| x.display.to_string()),
				)
			});
			reg.add_function("style", |_, me: AnyUserData| {
				let manager = me.named_user_value::<UserDataRef<yazi_core::manager::Manager>>("manager")?;
				let file = me.borrow::<yazi_core::files::File>()?;
				let mime = manager.mimetype.get(&file.url);
				Ok(
					THEME
						.filetypes
						.iter()
						.find(|&x| x.matches(&file.url, mime, file.is_dir()))
						.map(|x| Style::from(x.style)),
				)
			});
			reg.add_function("is_hovered", |_, me: AnyUserData| {
				let folder = me.named_user_value::<UserDataRef<yazi_core::tab::Folder>>("folder")?;
				let file = me.borrow::<yazi_core::files::File>()?;
				Ok(matches!(folder.hovered(), Some(f) if f.url == file.url))
			});
			reg.add_function("is_yanked", |_, me: AnyUserData| {
				let manager = me.named_user_value::<UserDataRef<yazi_core::manager::Manager>>("manager")?;
				let file = me.borrow::<yazi_core::files::File>()?;
				Ok(if !manager.yanked.1.contains(&file.url) {
					0u8
				} else if manager.yanked.0 {
					2u8
				} else {
					1u8
				})
			});
			reg.add_function("is_selected", |_, me: AnyUserData| {
				let manager = me.named_user_value::<UserDataRef<yazi_core::manager::Manager>>("manager")?;
				let folder = me.named_user_value::<UserDataRef<yazi_core::tab::Folder>>("folder")?;
				let file = me.borrow::<yazi_core::files::File>()?;

				let selected = folder.files.is_selected(&file.url);
				Ok(if !manager.active().mode.is_visual() {
					selected
				} else {
					let idx: usize = me.named_user_value("idx")?;
					manager.active().mode.pending(folder.offset + idx, selected)
				})
			});
			reg.add_function("found", |lua, me: AnyUserData| {
				let manager = me.named_user_value::<UserDataRef<yazi_core::manager::Manager>>("manager")?;
				let Some(finder) = &manager.active().finder else {
					return Ok(None);
				};

				let file = me.borrow::<yazi_core::files::File>()?;
				if let Some(idx) = finder.matched_idx(&file.url) {
					return Some(
						lua.create_sequence_from([idx.into_lua(lua)?, finder.matched().len().into_lua(lua)?]),
					)
					.transpose();
				}
				Ok(None)
			});
			reg.add_function("highlights", |_, me: AnyUserData| {
				let manager = me.named_user_value::<UserDataRef<yazi_core::manager::Manager>>("manager")?;
				let Some(finder) = &manager.active().finder else {
					return Ok(None);
				};

				let file = me.borrow::<yazi_core::files::File>()?;
				let Some(h) = file.name().and_then(|n| finder.highlighted(n)) else {
					return Ok(None);
				};

				Ok(Some(h.into_iter().map(Range::from).collect::<Vec<_>>()))
			});
		})?;

		Ok(())
	}
}
