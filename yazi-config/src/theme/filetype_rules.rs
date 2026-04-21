use std::ops::Deref;

use arc_swap::ArcSwap;
use serde::Deserialize;
use yazi_codegen::Overlay;
use yazi_shim::arc_swap::IntoPointee;

use crate::theme::FiletypeRule;

#[derive(Default, Deserialize, Overlay)]
pub struct FiletypeRules(ArcSwap<Vec<FiletypeRule>>);

impl Deref for FiletypeRules {
	type Target = ArcSwap<Vec<FiletypeRule>>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl From<Vec<FiletypeRule>> for FiletypeRules {
	fn from(inner: Vec<FiletypeRule>) -> Self { Self(inner.into_pointee()) }
}
