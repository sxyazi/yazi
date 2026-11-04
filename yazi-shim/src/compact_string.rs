use std::{borrow::{Borrow, Cow}, fmt, ops::{Deref, DerefMut, RangeBounds}};

use compact_str::ToCompactString;
use mlua::{FromLua, IntoLua, Lua, LuaString, Value};
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Deserialize, Eq, Hash, Ord, PartialOrd, Serialize)]
#[repr(transparent)]
#[serde(transparent)]
pub struct CompactString(compact_str::CompactString);

impl Deref for CompactString {
	type Target = str;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl DerefMut for CompactString {
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl Borrow<str> for CompactString {
	fn borrow(&self) -> &str { self }
}

impl<T: ?Sized> AsRef<T> for CompactString
where
	compact_str::CompactString: AsRef<T>,
{
	fn as_ref(&self) -> &T { self.0.as_ref() }
}

impl<T: AsRef<str> + ?Sized> PartialEq<T> for CompactString {
	fn eq(&self, other: &T) -> bool { self.as_str() == other.as_ref() }
}

impl fmt::Display for CompactString {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { self.0.fmt(f) }
}

impl fmt::Debug for CompactString {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { self.0.fmt(f) }
}

impl From<&Self> for CompactString {
	fn from(value: &Self) -> Self { value.clone() }
}

impl<T> From<T> for CompactString
where
	T: Into<compact_str::CompactString>,
{
	fn from(value: T) -> Self { Self(value.into()) }
}

impl From<CompactString> for String {
	fn from(value: CompactString) -> Self { value.into_string() }
}

impl From<CompactString> for Cow<'_, str> {
	fn from(value: CompactString) -> Self { value.0.into() }
}

impl<'a> From<&'a CompactString> for Cow<'a, str> {
	fn from(value: &'a CompactString) -> Self { Self::Borrowed(value) }
}

impl FromIterator<char> for CompactString {
	fn from_iter<T: IntoIterator<Item = char>>(iter: T) -> Self { Self(iter.into_iter().collect()) }
}

impl CompactString {
	pub fn new(text: impl AsRef<str>) -> Self { Self(compact_str::CompactString::new(text)) }

	pub fn from_display(value: impl fmt::Display) -> Self { Self(value.to_compact_string()) }

	pub fn from_utf8_lossy(bytes: &[u8]) -> Self {
		Self(compact_str::CompactString::from_utf8_lossy(bytes))
	}

	pub fn as_str(&self) -> &str { self.0.as_str() }

	pub fn into_ascii_lowercase(mut self) -> Self {
		self.make_ascii_lowercase();
		self
	}

	pub fn into_string(self) -> String { self.0.into_string() }

	pub fn is_empty(&self) -> bool { self.0.is_empty() }

	pub fn clear(&mut self) { self.0.clear(); }

	pub fn push(&mut self, c: char) { self.0.push(c); }

	pub fn truncate(&mut self, len: usize) { self.0.truncate(len); }

	pub fn drain(&mut self, range: impl RangeBounds<usize>) -> compact_str::Drain<'_> {
		self.0.drain(range)
	}
}

impl FromLua for CompactString {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		Ok(Self::new(&*LuaString::from_lua(value, lua)?.to_str()?))
	}
}

impl IntoLua for CompactString {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		if self.0.is_heap_allocated() {
			lua.create_external_string(self.into_string())?.into_lua(lua)
		} else {
			lua.create_string(self)?.into_lua(lua)
		}
	}
}

impl IntoLua for &CompactString {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> { lua.create_string(self)?.into_lua(lua) }
}
