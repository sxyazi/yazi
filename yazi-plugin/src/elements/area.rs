use std::fmt::Debug;

use mlua::{AnyUserData, ExternalError, FromLua, IntoLua, Value};

use super::{Pos, Rect};

const EXPECTED: &str = "expected a Pos or Rect";

#[derive(Clone, Copy)]
pub enum Area {
	Pos(Pos),
	Rect(Rect),
}

impl Default for Area {
	fn default() -> Self { Self::Rect(Default::default()) }
}

impl Area {
	#[inline]
	pub fn size(self) -> ratatui::layout::Size {
		match self {
			Self::Pos(pos) => {
				ratatui::layout::Size { width: pos.offset.width, height: pos.offset.height }
			}
			Self::Rect(rect) => (*rect).into(),
		}
	}

	#[inline]
	pub fn inner(self, padding: ratatui::widgets::Padding) -> Self {
		match self {
			Self::Pos(mut pos) => {
				pos.pad += padding;
				Self::Pos(pos)
			}
			Self::Rect(rect) => Self::Rect(rect.pad(padding.into())),
		}
	}

	#[inline]
	pub fn transform(
		self,
		f: impl FnOnce(yazi_config::popup::Position) -> ratatui::layout::Rect,
	) -> ratatui::layout::Rect {
		match self {
			Self::Pos(pos) => *Rect(f(*pos)).pad(pos.pad),
			Self::Rect(rect) => *rect,
		}
	}
}

impl TryFrom<AnyUserData> for Area {
	type Error = mlua::Error;

	fn try_from(value: AnyUserData) -> Result<Self, Self::Error> {
		Ok(if let Ok(rect) = value.borrow::<Rect>() {
			Self::Rect(*rect)
		} else if let Ok(pos) = value.borrow::<Pos>() {
			Self::Pos(*pos)
		} else {
			return Err(EXPECTED.into_lua_err());
		})
	}
}

impl FromLua for Area {
	fn from_lua(value: Value, _: &mlua::Lua) -> mlua::Result<Self> {
		match value {
			Value::UserData(ud) => Self::try_from(ud),
			_ => Err(EXPECTED.into_lua_err()),
		}
	}
}

impl IntoLua for Area {
	fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
		match self {
			Self::Pos(pos) => pos.into_lua(lua),
			Self::Rect(rect) => rect.into_lua(lua),
		}
	}
}

impl Debug for Area {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Pos(pos) => write!(f, "{:?}", **pos),
			Self::Rect(rect) => write!(f, "{:?}", **rect),
		}
	}
}
