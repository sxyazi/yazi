use serde::{Deserialize, Deserializer};
use yazi_shared::Layer;

use super::Control;
use crate::{Preset, MERGED_KEYMAP};

#[derive(Debug)]
pub struct Keymap {
	pub manager:    Vec<Control>,
	pub tasks:      Vec<Control>,
	pub select:     Vec<Control>,
	pub input:      Vec<Control>,
	pub help:       Vec<Control>,
	pub completion: Vec<Control>,
}

impl<'de> Deserialize<'de> for Keymap {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		#[derive(Deserialize)]
		struct Shadow {
			manager:    Inner,
			tasks:      Inner,
			select:     Inner,
			input:      Inner,
			help:       Inner,
			completion: Inner,
		}
		#[derive(Deserialize)]
		struct Inner {
			keymap:         Vec<Control>,
			#[serde(default)]
			prepend_keymap: Vec<Control>,
			#[serde(default)]
			append_keymap:  Vec<Control>,
		}

		let mut shadow = Shadow::deserialize(deserializer)?;

		#[rustfmt::skip]
		Preset::mix(&mut shadow.manager.keymap, shadow.manager.prepend_keymap, shadow.manager.append_keymap);
		#[rustfmt::skip]
		Preset::mix(&mut shadow.tasks.keymap, shadow.tasks.prepend_keymap, shadow.tasks.append_keymap);
		#[rustfmt::skip]
		Preset::mix(&mut shadow.select.keymap, shadow.select.prepend_keymap, shadow.select.append_keymap);
		#[rustfmt::skip]
		Preset::mix(&mut shadow.input.keymap, shadow.input.prepend_keymap, shadow.input.append_keymap);
		#[rustfmt::skip]
		Preset::mix(&mut shadow.help.keymap, shadow.help.prepend_keymap, shadow.help.append_keymap);
		#[rustfmt::skip]
		Preset::mix(&mut shadow.completion.keymap, shadow.completion.prepend_keymap, shadow.completion.append_keymap);

		// TODO: remove this when v0.2.3 is released --
		if !shadow.input.keymap.iter().any(|c| c.on() == "<Backspace>") {
			println!(
				"WARNING: Default keybinding for `<Backspace>` is missing, please add a `{}` to the `[input]` section of `keymap.toml`.
In Yazi v0.2.0, `<Backspace>` previously hardcoded within the input component has been moved to `keymap.toml` to allow users to customize it.",
				r#"{ on = [ "<Backspace>" ], exec = "backspace" }"#
			);
		}
		// TODO: -- remove this when v0.2.3 is released

		// TODO: remove this when v0.2.3 is released --
		if shadow.manager.keymap.iter().any(|c| c.exec().contains("--empty=name")) {
			println!(
				"WARNING: `rename --empty=name` is deprecated in Yazi v0.2.2, please use `rename --empty=stem` instead.",
			);
		}
		// TODO: -- remove this when v0.2.3 is released

		Ok(Self {
			manager:    shadow.manager.keymap,
			tasks:      shadow.tasks.keymap,
			select:     shadow.select.keymap,
			input:      shadow.input.keymap,
			help:       shadow.help.keymap,
			completion: shadow.completion.keymap,
		})
	}
}

impl Default for Keymap {
	fn default() -> Self { toml::from_str(&MERGED_KEYMAP).unwrap() }
}

impl Keymap {
	#[inline]
	pub fn get(&self, layer: Layer) -> &Vec<Control> {
		match layer {
			Layer::App => unreachable!(),
			Layer::Manager => &self.manager,
			Layer::Tasks => &self.tasks,
			Layer::Select => &self.select,
			Layer::Input => &self.input,
			Layer::Help => &self.help,
			Layer::Completion => &self.completion,
			Layer::Which => unreachable!(),
		}
	}
}
