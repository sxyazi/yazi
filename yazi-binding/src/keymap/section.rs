use std::ops::Deref;

use anyhow::bail;
use mlua::{UserData, UserDataFields};
use yazi_config::KEYMAP;
use yazi_shim::mlua::UserDataFieldsExt;

use crate::keymap::Chords;

pub struct KeymapSection {
	inner: &'static yazi_config::keymap::KeymapSection,
	layer: yazi_shared::Layer,
}

impl Deref for KeymapSection {
	type Target = yazi_config::keymap::KeymapSection;

	fn deref(&self) -> &Self::Target { self.inner }
}

impl TryFrom<yazi_shared::Layer> for KeymapSection {
	type Error = anyhow::Error;

	fn try_from(layer: yazi_shared::Layer) -> Result<Self, Self::Error> {
		use yazi_shared::Layer as L;

		let inner = match layer {
			L::Null | L::App => bail!("invalid layer"),
			L::Mgr => KEYMAP.mgr.as_erased(),
			L::Tasks => KEYMAP.tasks.as_erased(),
			L::Spot => KEYMAP.spot.as_erased(),
			L::Pick => KEYMAP.pick.as_erased(),
			L::Input => KEYMAP.input.as_erased(),
			L::Confirm => KEYMAP.confirm.as_erased(),
			L::Help => KEYMAP.help.as_erased(),
			L::Cmp => KEYMAP.cmp.as_erased(),
			L::Which => bail!("invalid layer"),
			L::Notify => bail!("invalid layer"),
		};

		Ok(Self { inner, layer })
	}
}

impl UserData for KeymapSection {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_cached_field("rules", |_, me| Ok(Chords { inner: me.inner, layer: me.layer }));
	}
}
