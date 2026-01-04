use mlua::{AnyUserData, Function, Lua, Table};
use yazi_binding::elements::{Edge, Renderable};
use yazi_config::THEME;
use yazi_parser::mgr::{SpotLock, UpdateSpottedOpt};
use yazi_proxy::MgrProxy;

use super::Utils;

impl Utils {
	pub(super) fn spot_table(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|_, (t, table): (mlua::Table, AnyUserData)| {
			let mut lock = SpotLock::try_from(t)?;
			let mut table = yazi_binding::elements::Table::try_from(table)?;

			let area = table.area;
			table.area = area.inner(ratatui::widgets::Padding::uniform(1));

			lock.data = vec![
				Renderable::Clear(yazi_binding::elements::Clear { area }),
				Renderable::Border(yazi_binding::elements::Border {
					area,
					edge: Edge(ratatui::widgets::Borders::ALL),
					r#type: ratatui::widgets::BorderType::Rounded,
					style: THEME.spot.border.into(),
					titles: vec![(
						ratatui::widgets::TitlePosition::Top,
						ratatui::text::Line::raw("Spot").centered().style(THEME.spot.title),
					)],
				}),
				Renderable::Table(Box::new(table)),
			];
			MgrProxy::update_spotted(UpdateSpottedOpt { lock });

			Ok(())
		})
	}

	pub(super) fn spot_widgets(lua: &Lua) -> mlua::Result<Function> {
		lua.create_function(|_, (t, widgets): (Table, Vec<AnyUserData>)| {
			let mut lock = SpotLock::try_from(t)?;
			lock.data = widgets.into_iter().map(Renderable::try_from).collect::<mlua::Result<_>>()?;

			MgrProxy::update_spotted(UpdateSpottedOpt { lock });
			Ok(())
		})
	}
}
