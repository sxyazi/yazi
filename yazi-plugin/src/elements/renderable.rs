use mlua::{AnyUserData, ExternalError};

use super::{Bar, Border, Clear, Gauge, Line, List, Table, Text};

#[derive(Clone, Debug)]
pub enum Renderable {
	Line(Line),
	Text(Text),
	List(List),
	Bar(Bar),
	Clear(Clear),
	Border(Border),
	Gauge(Gauge),
	Table(Table),
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

impl TryFrom<AnyUserData> for Renderable {
	type Error = mlua::Error;

	fn try_from(ud: AnyUserData) -> Result<Self, Self::Error> {
		Ok(if let Ok(c) = ud.take::<crate::elements::Line>() {
			Self::Line(c)
		} else if let Ok(c) = ud.take::<crate::elements::Text>() {
			Self::Text(c)
		} else if let Ok(c) = ud.take::<crate::elements::List>() {
			Self::List(c)
		} else if let Ok(c) = ud.take::<crate::elements::Bar>() {
			Self::Bar(c)
		} else if let Ok(c) = ud.take::<crate::elements::Clear>() {
			Self::Clear(c)
		} else if let Ok(c) = ud.take::<crate::elements::Border>() {
			Self::Border(c)
		} else if let Ok(c) = ud.take::<crate::elements::Gauge>() {
			Self::Gauge(c)
		} else if let Ok(c) = ud.take::<crate::elements::Table>() {
			Self::Table(c)
		} else {
			return Err(
				format!("expected a UserData of renderable element, not: {ud:#?}").into_lua_err(),
			);
		})
	}
}
