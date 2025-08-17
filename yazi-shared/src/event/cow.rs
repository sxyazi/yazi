use std::{borrow::Cow, ops::Deref};

use anyhow::Result;

use super::{Cmd, Data, DataKey};
use crate::{SStr, url::UrlCow};

#[derive(Debug)]
pub enum CmdCow {
	Owned(Cmd),
	Borrowed(&'static Cmd),
}

impl From<Cmd> for CmdCow {
	fn from(c: Cmd) -> Self { Self::Owned(c) }
}

impl From<&'static Cmd> for CmdCow {
	fn from(c: &'static Cmd) -> Self { Self::Borrowed(c) }
}

impl Deref for CmdCow {
	type Target = Cmd;

	fn deref(&self) -> &Self::Target {
		match self {
			Self::Owned(c) => c,
			Self::Borrowed(c) => c,
		}
	}
}

impl CmdCow {
	#[inline]
	pub fn try_take(&mut self, name: impl Into<DataKey>) -> Option<Data> {
		match self {
			Self::Owned(c) => c.take(name),
			Self::Borrowed(_) => None,
		}
	}

	#[inline]
	pub fn take_str(&mut self, name: impl Into<DataKey>) -> Option<SStr> {
		match self {
			Self::Owned(c) => c.take_str(name),
			Self::Borrowed(c) => c.str(name).map(Cow::Borrowed),
		}
	}

	#[inline]
	pub fn take_url(&mut self, name: impl Into<DataKey>) -> Option<UrlCow<'static>> {
		match self {
			Self::Owned(c) => c.take(name).and_then(Data::into_url),
			Self::Borrowed(c) => c.url(name),
		}
	}

	#[inline]
	pub fn take_first_str(&mut self) -> Option<SStr> {
		match self {
			Self::Owned(c) => c.take_first_str(),
			Self::Borrowed(c) => c.first_str().map(Cow::Borrowed),
		}
	}

	pub fn take_first_url(&mut self) -> Option<UrlCow<'static>> {
		match self {
			Self::Owned(c) => c.take_first_url(),
			Self::Borrowed(c) => c.first().and_then(Data::to_url),
		}
	}

	#[inline]
	pub fn take_any<T: 'static>(&mut self, name: impl Into<DataKey>) -> Option<T> {
		match self {
			Self::Owned(c) => c.take_any(name),
			Self::Borrowed(_) => None,
		}
	}

	#[inline]
	pub fn take_any2<T: 'static>(&mut self, name: impl Into<DataKey>) -> Option<Result<T>> {
		match self {
			Self::Owned(c) => c.take_any2(name),
			Self::Borrowed(_) => None,
		}
	}
}
