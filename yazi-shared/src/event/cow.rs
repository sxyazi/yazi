use std::ops::Deref;

use anyhow::Result;

use super::Cmd;
use crate::data::{Data, DataKey};

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
	pub fn take<'a, T>(&mut self, name: impl Into<DataKey>) -> Result<T>
	where
		T: TryFrom<Data> + TryFrom<&'a Data>,
		<T as TryFrom<Data>>::Error: Into<anyhow::Error>,
		<T as TryFrom<&'a Data>>::Error: Into<anyhow::Error>,
	{
		match self {
			Self::Owned(c) => c.take(name),
			Self::Borrowed(c) => c.get(name),
		}
	}

	pub fn take_first<'a, T>(&mut self) -> Result<T>
	where
		T: TryFrom<Data> + TryFrom<&'a Data>,
		<T as TryFrom<Data>>::Error: Into<anyhow::Error>,
		<T as TryFrom<&'a Data>>::Error: Into<anyhow::Error>,
	{
		match self {
			Self::Owned(c) => c.take_first(),
			Self::Borrowed(c) => c.first(),
		}
	}

	pub fn take_second<'a, T>(&mut self) -> Result<T>
	where
		T: TryFrom<Data> + TryFrom<&'a Data>,
		<T as TryFrom<Data>>::Error: Into<anyhow::Error>,
		<T as TryFrom<&'a Data>>::Error: Into<anyhow::Error>,
	{
		match self {
			Self::Owned(c) => c.take_second(),
			Self::Borrowed(c) => c.second(),
		}
	}

	pub fn take_seq<'a, T>(&mut self) -> Vec<T>
	where
		T: TryFrom<Data> + TryFrom<&'a Data>,
		<T as TryFrom<Data>>::Error: Into<anyhow::Error>,
		<T as TryFrom<&'a Data>>::Error: Into<anyhow::Error>,
	{
		match self {
			Self::Owned(c) => c.take_seq(),
			Self::Borrowed(c) => c.seq(),
		}
	}

	pub fn take_any<T: 'static>(&mut self, name: impl Into<DataKey>) -> Option<T> {
		match self {
			Self::Owned(c) => c.take_any(name),
			Self::Borrowed(_) => None,
		}
	}

	pub fn take_any2<T: 'static>(&mut self, name: impl Into<DataKey>) -> Option<Result<T>> {
		match self {
			Self::Owned(c) => c.take_any2(name),
			Self::Borrowed(_) => None,
		}
	}
}
