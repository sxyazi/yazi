use std::any::TypeId;

use mlua::{AnyUserData, ExternalError};
use yazi_binding::Error;

use super::{Bar, Border, Clear, Gauge, Line, List, Table, Text};
use crate::elements::Rect;

#[derive(Clone, Debug)]
pub enum Renderable {
	Line(Line),
	Text(Text),
	List(Box<List>),
	Bar(Bar),
	Clear(Clear),
	Border(Border),
	Gauge(Box<Gauge>),
	Table(Box<Table>),
}

impl Renderable {
	pub fn render(
		self,
		buf: &mut ratatui::buffer::Buffer,
		trans: impl Fn(yazi_config::popup::Position) -> ratatui::layout::Rect,
	) {
		match self {
			Self::Line(line) => line.render(buf, trans),
			Self::Text(text) => text.render(buf, trans),
			Self::List(list) => list.render(buf, trans),
			Self::Bar(bar) => bar.render(buf, trans),
			Self::Clear(clear) => clear.render(buf, trans),
			Self::Border(border) => border.render(buf, trans),
			Self::Gauge(gauge) => gauge.render(buf, trans),
			Self::Table(table) => table.render(buf, trans),
		}
	}
}

impl TryFrom<&AnyUserData> for Renderable {
	type Error = mlua::Error;

	fn try_from(ud: &AnyUserData) -> Result<Self, Self::Error> {
		Ok(match ud.type_id() {
			Some(t) if t == TypeId::of::<Line>() => Self::Line(ud.take()?),
			Some(t) if t == TypeId::of::<Text>() => Self::Text(ud.take()?),
			Some(t) if t == TypeId::of::<List>() => Self::List(Box::new(ud.take()?)),
			Some(t) if t == TypeId::of::<Bar>() => Self::Bar(ud.take()?),
			Some(t) if t == TypeId::of::<Clear>() => Self::Clear(ud.take()?),
			Some(t) if t == TypeId::of::<Border>() => Self::Border(ud.take()?),
			Some(t) if t == TypeId::of::<Gauge>() => Self::Gauge(Box::new(ud.take()?)),
			Some(t) if t == TypeId::of::<Table>() => Self::Table(Box::new(ud.take()?)),
			_ => Err(format!("expected a UserData of renderable element, not: {ud:#?}").into_lua_err())?,
		})
	}
}

impl TryFrom<AnyUserData> for Renderable {
	type Error = mlua::Error;

	fn try_from(ud: AnyUserData) -> Result<Self, Self::Error> { Self::try_from(&ud) }
}

impl From<(Rect, Error)> for Renderable {
	fn from((area, error): (Rect, Error)) -> Self {
		Self::Text(Text {
			area: area.into(),
			inner: error.into_string().into(),
			wrap: ratatui::widgets::Wrap { trim: false }.into(),
			..Default::default()
		})
	}
}
