use std::{collections::HashSet, str::FromStr};

use anyhow::Context;
use indexmap::IndexSet;
use serde::{Deserialize, Deserializer};
use yazi_shared::Layer;

use super::Chord;
use crate::{Preset, keymap::Key};

#[derive(Debug)]
pub struct Keymap {
	pub mgr:     Vec<Chord>,
	pub tasks:   Vec<Chord>,
	pub spot:    Vec<Chord>,
	pub pick:    Vec<Chord>,
	pub input:   Vec<Chord>,
	pub confirm: Vec<Chord>,
	pub help:    Vec<Chord>,
	pub cmp:     Vec<Chord>,
}

impl Keymap {
	#[inline]
	pub fn get(&self, layer: Layer) -> &[Chord] {
		match layer {
			Layer::App => &[],
			Layer::Mgr => &self.mgr,
			Layer::Tasks => &self.tasks,
			Layer::Spot => &self.spot,
			Layer::Pick => &self.pick,
			Layer::Input => &self.input,
			Layer::Confirm => &self.confirm,
			Layer::Help => &self.help,
			Layer::Cmp => &self.cmp,
			Layer::Which => &[],
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
			#[serde(rename = "manager")]
			mgr:     Inner, // TODO: remove serde(rename)
			tasks:   Inner,
			spot:    Inner,
			pick:    Inner,
			input:   Inner,
			confirm: Inner,
			help:    Inner,
			cmp:     Inner,
		}
		#[derive(Deserialize)]
		struct Inner {
			keymap:         IndexSet<Chord>,
			#[serde(default)]
			prepend_keymap: IndexSet<Chord>,
			#[serde(default)]
			append_keymap:  IndexSet<Chord>,
		}

		fn mix(l: Layer, a: IndexSet<Chord>, b: IndexSet<Chord>, c: IndexSet<Chord>) -> Vec<Chord> {
			#[inline]
			fn on(Chord { on, .. }: &Chord) -> [Key; 2] {
				[on.first().copied().unwrap_or_default(), on.get(1).copied().unwrap_or_default()]
			}

			let a_seen: HashSet<_> = a.iter().map(on).collect();
			let b_seen: HashSet<_> = b.iter().map(on).collect();

			Preset::mix(
				a,
				b.into_iter().filter(|v| !a_seen.contains(&on(v))),
				c.into_iter().filter(|v| !b_seen.contains(&on(v))),
			)
			.filter(|chord| !chord.noop())
			.map(|chord| chord.with_layer(l))
			.collect()
		}

		let shadow = Shadow::deserialize(deserializer)?;
		Ok(Self {
			#[rustfmt::skip]
			mgr:     mix(Layer::Mgr, shadow.mgr.prepend_keymap, shadow.mgr.keymap, shadow.mgr.append_keymap),
			#[rustfmt::skip]
			tasks:   mix(Layer::Tasks, shadow.tasks.prepend_keymap, shadow.tasks.keymap, shadow.tasks.append_keymap),
			#[rustfmt::skip]
			spot:    mix(Layer::Spot, shadow.spot.prepend_keymap, shadow.spot.keymap, shadow.spot.append_keymap),
			#[rustfmt::skip]
			pick:    mix(Layer::Pick, shadow.pick.prepend_keymap, shadow.pick.keymap, shadow.pick.append_keymap),
			#[rustfmt::skip]
			input:   mix(Layer::Input, shadow.input.prepend_keymap, shadow.input.keymap, shadow.input.append_keymap),
			#[rustfmt::skip]
			confirm: mix(Layer::Confirm, shadow.confirm.prepend_keymap, shadow.confirm.keymap, shadow.confirm.append_keymap),
			#[rustfmt::skip]
			help:    mix(Layer::Help, shadow.help.prepend_keymap, shadow.help.keymap, shadow.help.append_keymap),
			#[rustfmt::skip]
			cmp:     mix(Layer::Cmp, shadow.cmp.prepend_keymap, shadow.cmp.keymap, shadow.cmp.append_keymap),
		})
	}
}
