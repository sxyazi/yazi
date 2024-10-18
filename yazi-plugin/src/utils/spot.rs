use mlua::{AnyUserData, Lua, Table};
use yazi_macro::emit;
use yazi_shared::{Layer, event::Cmd};

use super::Utils;
use crate::{bindings::Window, cast_to_renderable, elements::Renderable, file::FileRef};

pub struct SpotLock {
	pub url:  yazi_shared::fs::Url,
	pub cha:  yazi_shared::fs::Cha,
	pub mime: String,

	pub skip:   usize,
	pub window: Window,
	pub data:   Vec<Box<dyn Renderable + Send>>,
}

impl<'a> TryFrom<Table<'a>> for SpotLock {
	type Error = mlua::Error;

	fn try_from(t: Table) -> Result<Self, Self::Error> {
		let file: FileRef = t.raw_get("_file")?;
		Ok(Self {
			url:  file.url_owned(),
			cha:  file.cha,
			mime: t.raw_get("_mime")?,

			skip:   t.raw_get("_skip")?,
			window: t.raw_get("_window")?,
			data:   Default::default(),
		})
	}
}

impl Utils {
	pub(super) fn spot(lua: &Lua, ya: &Table) -> mlua::Result<()> {
		ya.raw_set(
			"spot_widgets",
			lua.create_async_function(|_, (t, widgets): (Table, Vec<AnyUserData>)| async move {
				let mut lock = SpotLock::try_from(t)?;
				lock.data = widgets.into_iter().filter_map(|ud| cast_to_renderable(&ud)).collect();

				emit!(Call(Cmd::new("update_spotted").with_any("lock", lock), Layer::Manager));
				Ok(())
			})?,
		)?;

		Ok(())
	}
}
