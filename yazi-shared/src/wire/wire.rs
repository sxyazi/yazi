use std::{collections::BTreeMap, fmt};

use anyhow::Result;
use mlua::{FromLua, Lua, Value};
use ordered_float::OrderedFloat;

use super::{Decoder, Encoder, WireKey};
use crate::{id::Id, sendable::Sendable, url::UrlBuf};

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub enum Wire {
	#[default]
	Nil,
	Boolean(bool),
	Integer(i64),
	Number(OrderedFloat<f64>),
	String(String),
	List(Vec<Self>),
	Dict(BTreeMap<WireKey, Self>),
	Id(Id),
	Url(UrlBuf),
	Bytes(Vec<u8>),
}

impl From<()> for Wire {
	fn from(_: ()) -> Self { Self::Nil }
}

impl From<bool> for Wire {
	fn from(value: bool) -> Self { Self::Boolean(value) }
}

impl From<i64> for Wire {
	fn from(value: i64) -> Self { Self::Integer(value) }
}

impl From<f64> for Wire {
	fn from(value: f64) -> Self { Self::Number(value.into()) }
}

impl From<String> for Wire {
	fn from(value: String) -> Self { Self::String(value) }
}

impl From<Vec<Self>> for Wire {
	fn from(value: Vec<Self>) -> Self { Self::List(value) }
}

impl From<BTreeMap<WireKey, Self>> for Wire {
	fn from(value: BTreeMap<WireKey, Self>) -> Self { Self::Dict(value) }
}

impl From<Id> for Wire {
	fn from(value: Id) -> Self { Self::Id(value) }
}

impl From<UrlBuf> for Wire {
	fn from(value: UrlBuf) -> Self { Self::Url(value) }
}

impl From<Vec<u8>> for Wire {
	fn from(value: Vec<u8>) -> Self { Self::Bytes(value) }
}

impl Wire {
	pub(crate) fn encode(&self, writer: &mut impl fmt::Write) -> fmt::Result {
		Encoder::new(writer).encode(self)
	}

	pub(crate) fn decode(bytes: &[u8]) -> Result<Self> { Decoder::new(bytes).decode() }
}

impl FromLua for Wire {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> { Sendable::value_to_wire(lua, value) }
}
