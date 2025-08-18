use std::any::TypeId;

use mlua::{AnyUserData, ExternalError};

use super::{Bar, Border, Clear, Gauge, Line, List, Table, Text};
use crate::{Error, elements::Area};

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
	pub fn area(&self) -> Area {
		match self {
			Self::Line(line) => line.area,
			Self::Text(text) => text.area,
			Self::List(list) => list.area,
			Self::Bar(bar) => bar.area,
			Self::Clear(clear) => clear.area,
			Self::Border(border) => border.area,
			Self::Gauge(gauge) => gauge.area,
			Self::Table(table) => table.area,
		}
	}

	pub fn with_area(mut self, area: impl Into<Area>) -> Self {
		let area = area.into();
		match &mut self {
			Self::Line(line) => line.area = area,
			Self::Text(text) => text.area = area,
			Self::List(list) => list.area = area,
			Self::Bar(bar) => bar.area = area,
			Self::Clear(clear) => clear.area = area,
			Self::Border(border) => border.area = area,
			Self::Gauge(gauge) => gauge.area = area,
			Self::Table(table) => table.area = area,
		}
		self
	}

	pub fn render(self, rect: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
		match self {
			Self::Line(line) => line.render(rect, buf),
			Self::Text(text) => text.render(rect, buf),
			Self::List(list) => list.render(rect, buf),
			Self::Bar(bar) => bar.render(rect, buf),
			Self::Clear(clear) => clear.render(rect, buf),
			Self::Border(border) => border.render(rect, buf),
			Self::Gauge(gauge) => gauge.render(rect, buf),
			Self::Table(table) => table.render(rect, buf),
		}
	}

	pub fn render_with<T>(self, buf: &mut ratatui::buffer::Buffer, trans: T)
	where
		T: FnOnce(yazi_config::popup::Position) -> ratatui::layout::Rect,
	{
		let rect = self.area().transform(trans);
		self.render(rect, buf);
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

impl From<Error> for Renderable {
	fn from(error: Error) -> Self {
		Self::Text(Text {
			inner: error.into_string().into(),
			wrap: ratatui::widgets::Wrap { trim: false }.into(),
			..Default::default()
		})
	}
}
