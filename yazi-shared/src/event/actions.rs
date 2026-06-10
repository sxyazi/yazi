use std::{ops::{Deref, DerefMut}, slice, vec};

use serde::{Deserializer, de};
use serde_with::{DeserializeAs, DisplayFromStr, OneOrMany};

use crate::{Layer, Source, event::Action};

#[derive(Clone, Debug, Default)]
pub struct Actions(Vec<Action>);

impl Deref for Actions {
	type Target = Vec<Action>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl DerefMut for Actions {
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl From<Action> for Actions {
	fn from(value: Action) -> Self { Self(vec![value]) }
}

impl From<Vec<Action>> for Actions {
	fn from(value: Vec<Action>) -> Self { Self(value) }
}

impl Actions {
	pub fn set(&mut self, layer: Layer, source: Source) {
		for action in &mut self.0 {
			action.source = source;
			if action.layer == Layer::Null {
				action.layer = layer;
			}
		}
	}
}

impl IntoIterator for Actions {
	type IntoIter = vec::IntoIter<Action>;
	type Item = Action;

	fn into_iter(self) -> Self::IntoIter { self.0.into_iter() }
}

impl<'a> IntoIterator for &'a Actions {
	type IntoIter = slice::Iter<'a, Action>;
	type Item = &'a Action;

	fn into_iter(self) -> Self::IntoIter { self.0.iter() }
}

pub fn deserialize_actions<'de, const L: u8, D>(deserializer: D) -> Result<Actions, D::Error>
where
	D: Deserializer<'de>,
{
	let Some(layer) = Layer::from_repr(L) else {
		return Err(de::Error::custom(format!("invalid keymap layer const: {L}")));
	};

	let mut actions = Actions(OneOrMany::<DisplayFromStr>::deserialize_as(deserializer)?);
	actions.set(layer, Source::Key);

	Ok(actions)
}
