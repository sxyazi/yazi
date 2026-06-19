use std::fmt::Debug;

use mlua::{AnyUserData, ExternalError, FromLua, IntoLua, Lua, Value};
use yazi_shim::ratatui::Padable;

use super::Rect;
use crate::position::Position;

const EXPECTED: &str = "expected a Pos or Rect";

#[derive(Clone, Copy)]
pub enum Area {
	Rect(Rect),
	Pos(Position),
}

impl Default for Area {
	fn default() -> Self { Self::Rect(Default::default()) }
}

impl Debug for Area {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Rect(rect) => write!(f, "{:?}", **rect),
			Self::Pos(pos) => write!(f, "{:?}", **pos),
		}
	}
}

impl From<Area> for Position {
	fn from(value: Area) -> Self {
		match value {
			Area::Rect(rect) => Self::from(*rect),
			Area::Pos(pos) => pos,
		}
	}
}

impl Area {
	pub fn size(self) -> ratatui_core::layout::Size {
		match self {
			Self::Rect(rect) => (*rect).into(),
			Self::Pos(pos) => ratatui_core::layout::Size { width: pos.width, height: pos.height },
		}
	}

	pub fn padding(self, padding: ratatui_widgets::block::Padding) -> Self {
		match self {
			Self::Rect(rect) => Self::Rect(rect.padding(padding).into()),
			Self::Pos(pos) => Self::Pos(pos.padding(padding)),
		}
	}

	pub fn transform(
		self,
		f: impl FnOnce(Position) -> ratatui_core::layout::Rect,
	) -> ratatui_core::layout::Rect {
		match self {
			Self::Rect(rect) => *rect,
			Self::Pos(pos) => f(pos),
		}
	}
}

impl From<Rect> for Area {
	fn from(rect: Rect) -> Self { Self::Rect(rect) }
}

impl From<ratatui_core::layout::Rect> for Area {
	fn from(rect: ratatui_core::layout::Rect) -> Self { Self::Rect(rect.into()) }
}

impl From<Position> for Area {
	fn from(value: Position) -> Self { Self::Pos(value.into()) }
}

impl TryFrom<&AnyUserData> for Area {
	type Error = mlua::Error;

	fn try_from(value: &AnyUserData) -> Result<Self, Self::Error> {
		Ok(if let Ok(rect) = value.borrow::<Rect>() {
			Self::Rect(*rect)
		} else if let Ok(pos) = value.borrow::<Position>() {
			Self::Pos(*pos)
		} else {
			return Err(EXPECTED.into_lua_err());
		})
	}
}

impl FromLua for Area {
	fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
		match value {
			Value::UserData(ud) => Self::try_from(&ud),
			_ => Err(EXPECTED.into_lua_err()),
		}
	}
}

impl IntoLua for Area {
	fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
		match self {
			Self::Rect(rect) => rect.into_lua(lua),
			Self::Pos(pos) => pos.into_lua(lua),
		}
	}
}
