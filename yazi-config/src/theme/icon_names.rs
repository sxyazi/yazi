use std::{ops::Deref, sync::Arc};

use arc_swap::ArcSwap;
use hashbrown::HashMap;
use serde::{Deserialize, Deserializer};
use yazi_codegen::Overlay;
use yazi_shared::{NonEmptyString, strand::AsStrand};
use yazi_shim::arc_swap::IntoPointee;

use crate::Icon;

#[derive(Default, Overlay)]
pub struct IconNames(ArcSwap<HashMap<NonEmptyString, Icon>>);

impl Deref for IconNames {
	type Target = ArcSwap<HashMap<NonEmptyString, Icon>>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl From<HashMap<NonEmptyString, Icon>> for IconNames {
	fn from(inner: HashMap<NonEmptyString, Icon>) -> Self { Self(inner.into_pointee()) }
}

impl IconNames {
	pub fn matches<S>(&self, name: S) -> Option<Icon>
	where
		S: AsStrand,
	{
		let name = name.as_strand().to_str().ok()?;
		if name.is_empty() {
			return None;
		}

		let inner = self.0.load();
		inner.get(name).or_else(|| inner.get(&name.to_ascii_lowercase())).cloned()
	}

	pub(super) fn unwrap_unchecked(self) -> HashMap<NonEmptyString, Icon> {
		Arc::try_unwrap(self.0.into_inner()).expect("unique icon names arc")
	}
}

impl<'de> Deserialize<'de> for IconNames {
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		deserialize_icon_names(deserializer).map(Self::from)
	}
}

pub(super) fn deserialize_icon_names<'de, D>(
	deserializer: D,
) -> Result<HashMap<NonEmptyString, Icon>, D::Error>
where
	D: Deserializer<'de>,
{
	#[derive(Deserialize)]
	struct Helper {
		name: NonEmptyString,
		#[serde(flatten)]
		icon: Icon,
	}

	Ok(
		Vec::<Helper>::deserialize(deserializer)?
			.into_iter()
			.map(|entry| (entry.name, entry.icon))
			.collect(),
	)
}
