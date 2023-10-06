use core::Ctx;

use config::{MANAGER, THEME};
use mlua::{AnyUserData, Function, IntoLua, MetaMethod, UserData, UserDataFields, UserDataMethods, Value};

use super::{Range, Url};
use crate::{layout::Style, LUA};

struct File(core::files::File);

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

pub struct Tab<'a, 'b> {
	scope: &'b mlua::Scope<'a, 'a>,

	cx:    &'a core::Ctx,
	inner: &'a core::manager::Tab,
}

impl<'a, 'b> Tab<'a, 'b> {
	pub(crate) fn init() -> mlua::Result<()> {
		LUA.register_userdata_type::<core::manager::Tab>(|reg| {
			reg.add_field_function_get("mode", |_, me| me.named_user_value::<AnyUserData>("mode"));
			reg.add_field_function_get("parent", |_, me| me.named_user_value::<Value>("parent"));
			reg.add_field_function_get("current", |_, me| me.named_user_value::<AnyUserData>("current"));
			reg.add_field_function_get("preview", |_, me| me.named_user_value::<AnyUserData>("preview"));
		})?;

		LUA.register_userdata_type::<core::manager::Mode>(|reg| {
			reg.add_field_method_get("is_select", |_, me| Ok(me.is_select()));
			reg.add_field_method_get("is_unset", |_, me| Ok(me.is_unset()));
			reg.add_field_method_get("is_visual", |_, me| Ok(me.is_visual()));
			reg.add_method("pending", |_, me, (idx, state): (usize, bool)| Ok(me.pending(idx, state)));

			reg.add_meta_method(MetaMethod::ToString, |_, me, ()| Ok(me.to_string()));
		})?;

		LUA.register_userdata_type::<core::manager::Folder>(|reg| {
			reg.add_field_method_get("cwd", |_, me| Ok(Url::from(&me.cwd)));
			reg.add_field_method_get("offset", |_, me| Ok(me.offset()));
			reg.add_field_method_get("cursor", |_, me| Ok(me.cursor()));

			reg.add_field_function_get("window", |_, me| me.named_user_value::<Value>("window"));
			reg.add_field_function_get("files", |_, me| me.named_user_value::<AnyUserData>("files"));
			reg.add_field_function_get("hovered", |_, me| me.named_user_value::<Value>("hovered"));
		})?;

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

		LUA.register_userdata_type::<core::manager::Preview>(|reg| {
			reg.add_field_function_get("folder", |_, me| me.named_user_value::<Value>("folder"));
		})?;

		Ok(())
	}

	pub(crate) fn new(
		scope: &'b mlua::Scope<'a, 'a>,

		cx: &'a Ctx,
		inner: &'a core::manager::Tab,
	) -> Self {
		Self { scope, cx, inner }
	}

	pub(crate) fn make(&self) -> mlua::Result<AnyUserData<'a>> {
		let ud = self.scope.create_any_userdata_ref(self.inner)?;
		ud.set_named_user_value("mode", self.scope.create_any_userdata_ref(&self.inner.mode)?)?;
		ud.set_named_user_value(
			"parent",
			self.inner.parent.as_ref().and_then(|p| self.folder(p, None).ok()),
		)?;
		ud.set_named_user_value("current", self.folder(&self.inner.current, None)?)?;
		ud.set_named_user_value("preview", self.preview(self.inner)?)?;

		Ok(ud)
	}

	pub(crate) fn folder(
		&self,
		inner: &'a core::manager::Folder,
		window: Option<(usize, usize)>,
	) -> mlua::Result<AnyUserData<'a>> {
		let window = window.unwrap_or_else(|| (inner.offset(), MANAGER.layout.folder_height()));

		let ud = self.scope.create_any_userdata_ref(inner)?;
		ud.set_named_user_value(
			"window",
			inner
				.files
				.iter()
				.skip(window.0)
				.take(window.1)
				.enumerate()
				.filter_map(|(i, f)| self.file(i, f, inner).ok())
				.collect::<Vec<_>>(),
		)?;
		ud.set_named_user_value("files", self.files(&inner.files)?)?;
		// TODO: remove this
		ud.set_named_user_value(
			"hovered",
			inner.hovered.as_ref().and_then(|h| self.file(999, h, inner).ok()),
		)?;

		Ok(ud)
	}

	fn files(&self, inner: &'a core::files::Files) -> mlua::Result<AnyUserData<'a>> {
		self.scope.create_any_userdata_ref(inner)
	}

	fn file(
		&self,
		idx: usize,
		inner: &'a core::files::File,
		folder: &'a core::manager::Folder,
	) -> mlua::Result<AnyUserData<'a>> {
		let ud = self.scope.create_any_userdata_ref(inner)?;
		ud.set_named_user_value("idx", idx)?;

		ud.set_named_user_value(
			"icon",
			self.scope.create_function(|_, ()| {
				Ok(
					THEME
						.icons
						.iter()
						.find(|&x| x.name.match_path(inner.url(), Some(inner.is_dir())))
						.map(|x| x.display.to_string()),
				)
			})?,
		)?;

		ud.set_named_user_value(
			"style",
			self.scope.create_function(|_, ()| {
				let mime = self.cx.manager.mimetype.get(inner.url());
				Ok(
					THEME
						.filetypes
						.iter()
						.find(|&x| x.matches(inner.url(), mime, inner.is_dir()))
						.map(|x| Style::from(x.style)),
				)
			})?,
		)?;

		ud.set_named_user_value(
			"hovered",
			matches!(&folder.hovered, Some(f) if f.url() == inner.url()),
		)?;

		ud.set_named_user_value(
			"selected",
			self.scope.create_function(|_, me: AnyUserData| {
				let is_visual = self.inner.mode.is_visual();
				let selected = folder.files.is_selected(inner.url());
				Ok(if !is_visual {
					selected
				} else {
					let idx: usize = me.named_user_value("idx")?;
					self.inner.mode.pending(folder.offset() + idx, selected)
				})
			})?,
		)?;

		ud.set_named_user_value(
			"highlights",
			self.scope.create_function(|_, ()| {
				let Some(finder) = self.inner.finder() else {
					return Ok(None);
				};
				Ok(
					inner
						.name()
						.map(|n| finder.highlighted(n).into_iter().map(Range::from).collect::<Vec<_>>()),
				)
			})?,
		)?;

		Ok(ud)
	}

	fn preview(&self, tab: &'a core::manager::Tab) -> mlua::Result<AnyUserData<'a>> {
		let inner = tab.preview();

		let ud = self.scope.create_any_userdata_ref(inner)?;
		ud.set_named_user_value(
			"folder",
			inner
				.lock
				.as_ref()
				.filter(|l| l.is_folder())
				.and_then(|l| tab.history(&l.url))
				.and_then(|f| self.folder(f, Some((f.offset(), MANAGER.layout.preview_height()))).ok()),
		)?;

		Ok(ud)
	}
}
