use std::{ops::Deref, sync::Arc};

use arc_swap::ArcSwap;
use serde::Deserialize;
use yazi_codegen::Overlay;
use yazi_fs::File;
use yazi_shim::arc_swap::IntoPointee;

use crate::{Icon, theme::IconCond};

#[derive(Default, Deserialize, Overlay)]
pub struct IconConds(ArcSwap<Vec<IconCond>>);

impl Deref for IconConds {
	type Target = ArcSwap<Vec<IconCond>>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl From<Vec<IconCond>> for IconConds {
	fn from(inner: Vec<IconCond>) -> Self { Self(inner.into_pointee()) }
}

impl IconConds {
	pub fn matches(&self, file: &File, hovered: bool) -> Option<Icon> {
		self.0.load().iter().find(|&c| c.matches(file, hovered)).map(|c| c.icon.clone())
	}

	pub(super) fn unwrap_unchecked(self) -> Vec<IconCond> {
		Arc::try_unwrap(self.0.into_inner()).expect("unique icon conds arc")
	}
}
