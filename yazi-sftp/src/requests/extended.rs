use std::{borrow::Cow, fmt::Debug};

use serde::{Deserialize, Serialize};

use crate::ByteStr;

#[derive(Debug, Deserialize, Serialize)]
pub struct Extended<'a, D> {
	pub id:      u32,
	pub request: Cow<'a, str>,
	pub data:    D,
}

impl<D: ExtendedData> Extended<'_, D> {
	pub fn new<'a, R>(request: R, data: D) -> Extended<'a, D>
	where
		R: Into<Cow<'a, str>>,
	{
		Extended { id: 0, request: request.into(), data }
	}

	pub fn len(&self) -> usize { size_of_val(&self.id) + 4 + self.request.len() + self.data.len() }
}

// --- Data
pub trait ExtendedData: Debug + Serialize + for<'de> Deserialize<'de> {
	fn len(&self) -> usize;
}

// --- Fsync
#[derive(Debug, Deserialize, Serialize)]
pub struct ExtendedFsync<'a> {
	pub handle: Cow<'a, str>,
}

impl<'a> ExtendedFsync<'a> {
	pub fn new(handle: impl Into<Cow<'a, str>>) -> Self { Self { handle: handle.into() } }
}

impl ExtendedData for ExtendedFsync<'_> {
	fn len(&self) -> usize { 4 + self.handle.len() }
}

// --- Hardlink
#[derive(Debug, Deserialize, Serialize)]
pub struct ExtendedHardlink<'a> {
	pub original: ByteStr<'a>,
	pub link:     ByteStr<'a>,
}

impl<'a> ExtendedHardlink<'a> {
	pub fn new<O, L>(original: O, link: L) -> Self
	where
		O: Into<ByteStr<'a>>,
		L: Into<ByteStr<'a>>,
	{
		Self { original: original.into(), link: link.into() }
	}
}

impl ExtendedData for ExtendedHardlink<'_> {
	fn len(&self) -> usize { 4 + self.original.len() + 4 + self.link.len() }
}

// --- Limits
#[derive(Debug, Deserialize, Serialize)]
pub struct ExtendedLimits;

impl ExtendedData for ExtendedLimits {
	fn len(&self) -> usize { 0 }
}
