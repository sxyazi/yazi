use mlua::{AnyUserData, Function, Lua, Table};
use yazi_config::THEME;
use yazi_macro::emit;
use yazi_shared::{Id, event::Cmd};

use super::Utils;
use crate::{elements::{Edge, Renderable}, file::FileRef};

pub struct SpotLock {
	pub url:  yazi_shared::url::Url,
	pub cha:  yazi_fs::cha::Cha,
	pub mime: String,

	pub id:   Id,
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

			id:   *t.raw_get::<yazi_binding::Id>("id")?,
			skip: t.raw_get("skip")?,
			data: Default::default(),
		})
	}
}

impl SpotLock {
	#[inline]
	pub fn len(&self) -> Option<usize> { Some(self.table()?.len()) }

	pub fn select(&mut self, idx: Option<usize>) {
		if let Some(t) = self.table_mut() {
			t.select(idx);
		}
	}

	pub fn selected(&self) -> Option<usize> { self.table()?.selected() }

	pub fn table(&self) -> Option<&crate::elements::Table> {
		self.data.iter().rev().find_map(|r| match r {
			Renderable::Table(t) => Some(t.as_ref()),
			_ => None,
		})
	}

	pub fn table_mut(&mut self) -> Option<&mut crate::elements::Table> {
		self.data.iter_mut().rev().find_map(|r| match r {
			Renderable::Table(t) => Some(t.as_mut()),
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
					edge: Edge(ratatui::widgets::Borders::ALL),
					r#type: ratatui::widgets::BorderType::Rounded,
					style: THEME.spot.border.into(),
					titles: vec![(
						ratatui::widgets::block::Position::Top,
						ratatui::text::Line::raw(lock.url.name().to_string_lossy().into_owned())
							.centered()
							.style(THEME.spot.title),
					)],
				}),
				Renderable::Table(Box::new(table)),
			];
			emit!(Call(Cmd::new("mgr:update_spotted").with_any("lock", lock)));

			Ok(())
		})
	}

	pub(super) fn spot_widgets(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|_, (t, widgets): (Table, Vec<AnyUserData>)| {
			let mut lock = SpotLock::try_from(t)?;
			lock.data = widgets.into_iter().map(Renderable::try_from).collect::<mlua::Result<_>>()?;

			emit!(Call(Cmd::new("mgr:update_spotted").with_any("lock", lock)));
			Ok(())
		})
	}
}
