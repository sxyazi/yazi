use mlua::{AnyUserData, Function, Lua, Table};
use yazi_config::THEME;
use yazi_macro::emit;
use yazi_shared::{Layer, event::Cmd};

use super::Utils;
use crate::{elements::Renderable, file::FileRef};

pub struct SpotLock {
	pub url:  yazi_shared::url::Url,
	pub cha:  yazi_fs::Cha,
	pub mime: String,

	pub skip: usize,
	pub data: Vec<Renderable>,
}

impl TryFrom<Table> for SpotLock {
	type Error = mlua::Error;

	fn try_from(t: Table) -> Result<Self, Self::Error> {
		let file: FileRef = t.raw_get("file")?;
		Ok(Self {
			url:  file.url_owned(),
			cha:  file.cha,
			mime: t.raw_get("mime")?,

			skip: t.raw_get("skip")?,
			data: Default::default(),
		})
	}
}

impl SpotLock {
	pub fn select(&mut self, idx: Option<usize>) {
		if let Some(t) = self.table_mut() {
			t.select(idx);
		}
	}

	pub fn selected(&self) -> Option<usize> { self.table()?.selected() }

	pub fn table(&self) -> Option<&crate::elements::Table> {
		self.data.iter().rev().find_map(|r| match r {
			Renderable::Table(t) => Some(t),
			_ => None,
		})
	}

	pub fn table_mut(&mut self) -> Option<&mut crate::elements::Table> {
		self.data.iter_mut().rev().find_map(|r| match r {
			Renderable::Table(t) => Some(t),
			_ => None,
		})
	}
}

impl Utils {
	pub(super) fn spot_table(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|_, (t, table): (mlua::Table, AnyUserData)| {
			let mut lock = SpotLock::try_from(t)?;
			let mut table = crate::elements::Table::try_from(table)?;

			let area = table.area;
			table.area = area.inner(ratatui::widgets::Padding::uniform(1));

			lock.data = vec![
				Renderable::Clear(crate::elements::Clear { area }),
				Renderable::Border(crate::elements::Border {
					area,
					position: ratatui::widgets::Borders::ALL,
					type_: ratatui::widgets::BorderType::Rounded,
					style: ratatui::style::Style::from(THEME.spot.border),
					titles: vec![(
						ratatui::widgets::block::Position::Top,
						ratatui::text::Line::raw(lock.url.name().to_string_lossy().into_owned()).centered().style(THEME.spot.title),
					)],
				}),
				Renderable::Table(table),
			];
			emit!(Call(Cmd::new("update_spotted").with_any("lock", lock), Layer::Manager));

			Ok(())
		})
	}

	pub(super) fn spot_widgets(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|_, (t, widgets): (Table, Vec<AnyUserData>)| {
			let mut lock = SpotLock::try_from(t)?;
			lock.data = widgets.into_iter().map(Renderable::try_from).collect::<mlua::Result<_>>()?;

			emit!(Call(Cmd::new("update_spotted").with_any("lock", lock), Layer::Manager));
			Ok(())
		})
	}
}
