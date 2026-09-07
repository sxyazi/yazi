use std::ops::Deref;

use serde::Deserialize;

use super::{AuthArc, View};
use crate::wire::Wire;

#[repr(transparent)]
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq)]
#[serde(transparent)]
pub struct ViewBox(Option<Box<View>>);

impl Deref for ViewBox {
	type Target = Option<Box<View>>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl Default for ViewBox {
	fn default() -> Self { Self::DEFAULT }
}

impl From<View> for ViewBox {
	fn from(view: View) -> Self { Self(Some(Box::new(view))) }
}

impl ViewBox {
	pub(crate) const DEFAULT: Self = Self(None);

	#[inline]
	pub fn as_ref(&self) -> Option<&View> { self.0.as_deref() }

	#[inline]
	pub fn auth(&self) -> Option<&AuthArc> { self.as_ref().map(|v| &v.source) }

	#[inline]
	pub fn data(&self) -> Option<&Wire> { self.as_ref().map(|v| &v.data) }

	#[inline]
	pub fn is_local(&self) -> bool { self.auth().is_some_and(|a| a.is_local()) }

	#[inline]
	pub fn is_remote(&self) -> bool { self.auth().is_some_and(|a| a.is_remote()) }
}
