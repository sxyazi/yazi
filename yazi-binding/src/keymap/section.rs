use std::ops::Deref;

use anyhow::bail;
use mlua::{UserData, UserDataFields, Value};
use yazi_config::KEYMAP;
use yazi_shared::Layer;

use crate::{cached_field, keymap::Chords};

pub struct KeymapSection {
	inner: &'static yazi_config::keymap::KeymapSection,

	v_rules: Option<Value>,
}

impl Deref for KeymapSection {
	type Target = yazi_config::keymap::KeymapSection;

	fn deref(&self) -> &Self::Target { self.inner }
}

impl TryFrom<Layer> for KeymapSection {
	type Error = anyhow::Error;

	fn try_from(value: Layer) -> Result<Self, Self::Error> {
		let inner = match value {
			Layer::Null | Layer::App => bail!("invalid layer"),
			Layer::Mgr => KEYMAP.mgr.as_erased(),
			Layer::Tasks => KEYMAP.tasks.as_erased(),
			Layer::Spot => KEYMAP.spot.as_erased(),
			Layer::Pick => KEYMAP.pick.as_erased(),
			Layer::Input => KEYMAP.input.as_erased(),
			Layer::Confirm => KEYMAP.confirm.as_erased(),
			Layer::Help => KEYMAP.help.as_erased(),
			Layer::Cmp => KEYMAP.cmp.as_erased(),
			Layer::Which => bail!("invalid layer"),
			Layer::Notify => bail!("invalid layer"),
		};

		Ok(Self { inner, v_rules: None })
	}
}

impl UserData for KeymapSection {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		cached_field!(fields, rules, |_, me| Ok(Chords::new(me.inner)));
	}
}
