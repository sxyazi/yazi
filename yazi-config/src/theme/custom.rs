use std::{fmt, mem, ops::Deref, sync::Arc};

use arc_swap::ArcSwap;
use hashbrown::{HashMap, hash_map};
use serde::{Deserialize, Deserializer, de::{MapAccess, Visitor}};
use yazi_codegen::{DeserializeOver, Overlay};
use yazi_shared::{KebabCasedString, SnakeCasedString};
use yazi_shim::{arc_swap::IntoPointee, toml::DeserializeOverWith};

use crate::theme::CustomSection;

#[derive(Debug, Default, DeserializeOver, Overlay)]
pub struct Custom(ArcSwap<HashMap<SnakeCasedString, CustomSection>>);

impl Deref for Custom {
	type Target = ArcSwap<HashMap<SnakeCasedString, CustomSection>>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl From<HashMap<SnakeCasedString, CustomSection>> for Custom {
	fn from(value: HashMap<SnakeCasedString, CustomSection>) -> Self { Self(value.into_pointee()) }
}

impl Custom {
	pub(super) fn unwrap_unchecked(self) -> HashMap<SnakeCasedString, CustomSection> {
		Arc::try_unwrap(self.0.into_inner()).expect("unique custom arc")
	}
}

impl<'de> Deserialize<'de> for Custom {
	fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
		struct V;

		impl<'de> Visitor<'de> for V {
			type Value = HashMap<SnakeCasedString, CustomSection>;

			fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result { f.write_str("a map") }

			fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Self::Value, M::Error> {
				let mut sections = HashMap::with_capacity(map.size_hint().unwrap_or(0));
				while let Some(key) = map.next_key::<KebabCasedString>()? {
					let section = map.next_value::<CustomSection>()?;
					if !section.load().is_empty() {
						sections.insert(key.into_snake_cased(), section);
					}
				}
				Ok(sections)
			}
		}

		Ok(de.deserialize_map(V)?.into())
	}
}

impl DeserializeOverWith for Custom {
	fn deserialize_over_with<'de, D: Deserializer<'de>>(self, de: D) -> Result<Self, D::Error> {
		struct V(Custom);

		impl<'de> Visitor<'de> for V {
			type Value = Custom;

			fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result { f.write_str("a map") }

			fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Custom, M::Error> {
				let mut sections = self.0.unwrap_unchecked();
				while let Some(key) = map.next_key::<KebabCasedString>()? {
					let (key, new) = (key.into_snake_cased(), map.next_value::<CustomSection>()?);
					match sections.entry(key) {
						hash_map::Entry::Occupied(mut oe) => {
							let mut old = mem::take(oe.get_mut()).unwrap_unchecked();
							old.extend(new.unwrap_unchecked());
							oe.insert(old.into());
						}
						hash_map::Entry::Vacant(_) if new.load().is_empty() => {}
						hash_map::Entry::Vacant(ve) => _ = ve.insert(new),
					}
				}
				Ok(sections.into())
			}
		}

		de.deserialize_map(V(self))
	}
}
