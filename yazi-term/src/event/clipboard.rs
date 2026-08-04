use std::collections::HashMap;

use compact_str::CompactString;
use mlua::{BString, IntoLua, Lua, Value};

use crate::parser::StateOsc5522;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ClipboardEvent {
	Read { primary: bool, pw: String, data: ClipboardData },
	ReadError(CompactString),
	WriteSuccess,
	WriteError(CompactString),
}

impl ClipboardEvent {
	pub fn r#type(&self) -> &'static str {
		match self {
			Self::Read { .. } => "read",
			Self::ReadError(_) | Self::WriteError(_) => "error",
			Self::WriteSuccess => "success",
		}
	}

	pub fn primary(&self) -> Option<bool> {
		match self {
			Self::Read { primary, .. } => Some(*primary),
			_ => None,
		}
	}

	pub fn pw(&mut self) -> Option<&mut String> {
		match self {
			Self::Read { pw, .. } => Some(pw),
			_ => None,
		}
	}

	pub fn data(&mut self) -> Option<&mut ClipboardData> {
		match self {
			Self::Read { data, .. } => Some(data),
			_ => None,
		}
	}

	pub fn is_write(&self) -> bool {
		match self {
			Self::WriteSuccess | Self::WriteError(_) => true,
			_ => false,
		}
	}

	pub(crate) fn from_state(s: StateOsc5522) -> Option<Self> {
		Some(match s {
			StateOsc5522 { write: false, status, .. } if status == "DONE" => Self::Read {
				primary: s.primary,
				pw:      s.pw,
				data:    ClipboardData(s.mimes.into_iter().zip(s.payload).collect()),
			},
			StateOsc5522 { write: false, status, .. } if !status.is_empty() => Self::ReadError(status),
			StateOsc5522 { write: true, status, .. } if status == "DONE" => Self::WriteSuccess,
			StateOsc5522 { write: true, status, .. } if !status.is_empty() => Self::WriteError(status),
			_ => return None,
		})
	}
}

// --- ClipboardData
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ClipboardData(HashMap<String, Vec<u8>>);

impl IntoLua for ClipboardData {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		lua
			.create_table_from(self.0.into_iter().map(|(mime, payload)| (mime, BString::new(payload))))?
			.into_lua(lua)
	}
}
