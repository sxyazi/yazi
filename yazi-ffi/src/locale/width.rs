use mlua::{BorrowedBytes, ExternalError, FromLua, Lua, Value};
use strum::EnumIs;
use yazi_shim::bytes::BytesExt;

#[derive(Clone, Copy, Default, EnumIs)]
pub(super) enum Width {
	#[default]
	None,
	Numeric,
	TwoDigit,
	System,
}

impl From<usize> for Width {
	fn from(len: usize) -> Self { if len == 2 { Self::TwoDigit } else { Self::Numeric } }
}

impl Width {
	pub(super) fn resolve(self, system: Self) -> Self { if self.is_system() { system } else { self } }

	pub(super) fn push(self, field: char, output: &mut String) {
		let (prefix, field) = match (self, field) {
			(Self::None, _) => return,
			(Self::System, _) => unreachable!("system width must be resolved before formatting"),
			(Self::Numeric, 'Y') => ("%", 'Y'),
			(Self::Numeric, field) => ("%-", field),
			(Self::TwoDigit, 'Y') => ("%", 'y'),
			(Self::TwoDigit, field) => ("%", field),
		};

		output.push_str(prefix);
		output.push(field);
	}
}

impl FromLua for Width {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		let value = BorrowedBytes::from_lua(value, lua)?;
		match &*value {
			b"numeric" => Ok(Self::Numeric),
			b"2-digit" => Ok(Self::TwoDigit),
			b"system" => Ok(Self::System),
			value => Err(format!("invalid date format width: {}", value.display()).into_lua_err()),
		}
	}
}
