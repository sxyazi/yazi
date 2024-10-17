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
	pub spot:       Vec<Chord>,
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
			Layer::Spot => &self.spot,
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
			spot:       Inner,
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

		fn mix(a: IndexSet<Chord>, b: IndexSet<Chord>, c: IndexSet<Chord>) -> Vec<Chord> {
			let a_seen: HashSet<_> =
				a.iter().filter(|&v| v.on.len() > 1).map(|v| [v.on[0], v.on[1]]).collect();
			let b_seen: HashSet<_> =
				b.iter().filter(|&v| v.on.len() > 1).map(|v| [v.on[0], v.on[1]]).collect();

			Preset::mix(
				a,
				b.into_iter().filter(|v| v.on.len() < 2 || !a_seen.contains(&v.on[..2])),
				c.into_iter().filter(|v| v.on.len() < 2 || !b_seen.contains(&v.on[..2])),
			)
			.filter(|c| !c.noop())
			.collect()
		}

		let shadow = Shadow::deserialize(deserializer)?;
		Ok(Self {
			#[rustfmt::skip]
			manager:    mix(shadow.manager.prepend_keymap, shadow.manager.keymap, shadow.manager.append_keymap),
			#[rustfmt::skip]
			tasks:      mix(shadow.tasks.prepend_keymap, shadow.tasks.keymap, shadow.tasks.append_keymap),
			#[rustfmt::skip]
			spot:       mix(shadow.spot.prepend_keymap, shadow.spot.keymap, shadow.spot.append_keymap),
			#[rustfmt::skip]
			pick:       mix(shadow.pick.prepend_keymap, shadow.pick.keymap, shadow.pick.append_keymap),
			#[rustfmt::skip]
			input:      mix(shadow.input.prepend_keymap, shadow.input.keymap, shadow.input.append_keymap),
			#[rustfmt::skip]
			confirm:    mix(shadow.confirm.prepend_keymap, shadow.confirm.keymap, shadow.confirm.append_keymap),
			#[rustfmt::skip]
			help:       mix(shadow.help.prepend_keymap, shadow.help.keymap, shadow.help.append_keymap),
			#[rustfmt::skip]
			completion: mix(shadow.completion.prepend_keymap, shadow.completion.keymap, shadow.completion.append_keymap),
		})
	}
}
