use std::{ops::Deref, sync::Arc};

use arc_swap::ArcSwap;
use serde::Deserialize;
use yazi_codegen::Overlay;
use yazi_fs::File;
use yazi_shim::arc_swap::IntoPointee;

use crate::{Icon, Selectable, theme::IconGlob};

#[derive(Default, Deserialize, Overlay)]
pub struct IconGlobs(ArcSwap<Vec<IconGlob>>);

impl Deref for IconGlobs {
	type Target = ArcSwap<Vec<IconGlob>>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl From<Vec<IconGlob>> for IconGlobs {
	fn from(inner: Vec<IconGlob>) -> Self { Self(inner.into_pointee()) }
}

impl IconGlobs {
	pub fn matches(&self, file: &File) -> Option<Icon> {
		self.0.load().iter().find(|&g| g.match_with(Some(file), None)).map(|g| g.icon.clone())
	}

	pub(super) fn unwrap_unchecked(self) -> Vec<IconGlob> {
		Arc::try_unwrap(self.0.into_inner()).expect("unique icon globs arc")
	}
}
