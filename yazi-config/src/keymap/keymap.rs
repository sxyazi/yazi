use std::{collections::HashSet, str::FromStr};

use anyhow::Context;
use indexmap::IndexSet;
use serde::{Deserialize, Deserializer};
use yazi_shared::Layer;

use super::Chord;
use crate::Preset;

#[derive(Debug)]
pub struct Keymap {
	pub manager:    Vec<Chord>,
	pub tasks:      Vec<Chord>,
	pub pick:       Vec<Chord>,
	pub input:      Vec<Chord>,
	pub confirm:    Vec<Chord>,
	pub help:       Vec<Chord>,
	pub completion: Vec<Chord>,
}

impl Keymap {
	#[inline]
	pub fn get(&self, layer: Layer) -> &[Chord] {
		match layer {
			Layer::App => unreachable!(),
			Layer::Manager => &self.manager,
			Layer::Tasks => &self.tasks,
			Layer::Pick => &self.pick,
			Layer::Input => &self.input,
			Layer::Confirm => &self.confirm,
			Layer::Help => &self.help,
			Layer::Completion => &self.completion,
			Layer::Which => unreachable!(),
		}
	}
}

impl FromStr for Keymap {
	type Err = anyhow::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		toml::from_str(s).context("Failed to parse your keymap.toml")
	}
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
			pick:       Inner,
			input:      Inner,
			confirm:    Inner,
			help:       Inner,
			completion: Inner,
		}
		#[derive(Deserialize)]
		struct Inner {
			keymap:         IndexSet<Chord>,
			#[serde(default)]
			prepend_keymap: IndexSet<Chord>,
			#[serde(default)]
			append_keymap:  IndexSet<Chord>,
		}

		fn mix(mut a: IndexSet<Chord>, b: IndexSet<Chord>, c: IndexSet<Chord>) -> Vec<Chord> {
			let mut seen = HashSet::new();
			b.iter().filter(|&v| v.on.len() > 1).for_each(|v| _ = seen.insert(&v.on[..2]));
			c.iter().filter(|&v| v.on.len() > 1).for_each(|v| _ = seen.insert(&v.on[..2]));

			a.retain(|v| v.on.len() < 2 || !seen.contains(&v.on[..2]));
			Preset::mix(a, b, c).collect()
		}

		let shadow = Shadow::deserialize(deserializer)?;
		Ok(Self {
			#[rustfmt::skip]
			manager:    mix(shadow.manager.keymap, shadow.manager.prepend_keymap, shadow.manager.append_keymap),
			#[rustfmt::skip]
			tasks:      mix(shadow.tasks.keymap, shadow.tasks.prepend_keymap, shadow.tasks.append_keymap),
			#[rustfmt::skip]
			pick:     mix(shadow.pick.keymap, shadow.pick.prepend_keymap, shadow.pick.append_keymap),
			#[rustfmt::skip]
			input:      mix(shadow.input.keymap, shadow.input.prepend_keymap, shadow.input.append_keymap),
			#[rustfmt::skip]
			confirm:    mix(shadow.confirm.keymap, shadow.confirm.prepend_keymap, shadow.confirm.append_keymap),
			#[rustfmt::skip]
			help:       mix(shadow.help.keymap, shadow.help.prepend_keymap, shadow.help.append_keymap),
			#[rustfmt::skip]
			completion: mix(shadow.completion.keymap, shadow.completion.prepend_keymap, shadow.completion.append_keymap),
		})
	}
}
