use std::{ops::Deref, sync::Arc};

use arc_swap::ArcSwap;
use hashbrown::HashMap;
use serde::{Deserialize, Deserializer, de::{self, MapAccess, SeqAccess, Visitor}};
use yazi_codegen::{DeserializeOver, Overlay};
use yazi_shared::SnakeCasedString;
use yazi_shim::arc_swap::IntoPointee;

use crate::theme::CustomField;

#[derive(Debug, Default, DeserializeOver, Overlay)]
pub struct CustomSection(ArcSwap<HashMap<SnakeCasedString, CustomField>>);

impl Deref for CustomSection {
	type Target = ArcSwap<HashMap<SnakeCasedString, CustomField>>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl From<HashMap<SnakeCasedString, CustomField>> for CustomSection {
	fn from(value: HashMap<SnakeCasedString, CustomField>) -> Self { Self(value.into_pointee()) }
}

impl CustomSection {
	pub(super) fn unwrap_unchecked(self) -> HashMap<SnakeCasedString, CustomField> {
		Arc::try_unwrap(self.0.into_inner()).expect("unique custom section arc")
	}
}

impl<'de> Deserialize<'de> for CustomSection {
	fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
		struct V;

		impl<'de> Visitor<'de> for V {
			type Value = CustomSection;

			fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
				f.write_str("a style section table or a skippable scalar")
			}

			fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Self::Value, M::Error> {
				let mut fields = HashMap::with_capacity(map.size_hint().unwrap_or(0));
				while let Some(k) = map.next_key::<SnakeCasedString>()? {
					fields.insert(k, map.next_value::<CustomField>()?);
				}
				Ok(CustomSection(fields.into_pointee()))
			}

			fn visit_bool<E: de::Error>(self, _: bool) -> Result<Self::Value, E> {
				Ok(CustomSection::default())
			}

			fn visit_i64<E: de::Error>(self, _: i64) -> Result<Self::Value, E> {
				Ok(CustomSection::default())
			}

			fn visit_u64<E: de::Error>(self, _: u64) -> Result<Self::Value, E> {
				Ok(CustomSection::default())
			}

			fn visit_f64<E: de::Error>(self, _: f64) -> Result<Self::Value, E> {
				Ok(CustomSection::default())
			}

			fn visit_str<E: de::Error>(self, _: &str) -> Result<Self::Value, E> {
				Ok(CustomSection::default())
			}

			fn visit_bytes<E: de::Error>(self, _: &[u8]) -> Result<Self::Value, E> {
				Ok(CustomSection::default())
			}

			fn visit_none<E: de::Error>(self) -> Result<Self::Value, E> { Ok(CustomSection::default()) }

			fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> { Ok(CustomSection::default()) }

			fn visit_some<D2: Deserializer<'de>>(self, de: D2) -> Result<Self::Value, D2::Error> {
				CustomSection::deserialize(de)
			}

			fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
				while seq.next_element::<de::IgnoredAny>()?.is_some() {}
				Ok(CustomSection::default())
			}
		}

		de.deserialize_any(V)
	}
}
