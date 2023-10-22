use mlua::{AnyUserData, FromLua, Lua, Table, UserData, UserDataMethods, Value};
use ratatui::widgets::Widget;

use super::{Rect, Span, Style};
use crate::{GLOBALS, LUA};

#[derive(Clone, Default)]
pub(crate) struct Gauge {
	area: ratatui::layout::Rect,

	ratio:       f64,
	label:       Option<ratatui::text::Span<'static>>,
	style:       Option<ratatui::style::Style>,
	gauge_style: Option<ratatui::style::Style>,
}

impl Gauge {
	pub(crate) fn install() -> mlua::Result<()> {
		let ui: Table = GLOBALS.get("ui")?;
		ui.set(
			"Gauge",
			LUA.create_function(|_, area: Rect| Ok(Gauge { area: area.0, ..Default::default() }))?,
		)
	}

	pub(crate) fn render(self, buf: &mut ratatui::buffer::Buffer) {
		let mut gauge = ratatui::widgets::Gauge::default();

		gauge = gauge.ratio(self.ratio);
		if let Some(label) = self.label {
			gauge = gauge.label(label);
		}
		if let Some(style) = self.style {
			gauge = gauge.style(style);
		}
		if let Some(gauge_style) = self.gauge_style {
			gauge = gauge.gauge_style(gauge_style);
		}

		gauge.render(self.area, buf)
	}
}

impl<'lua> FromLua<'lua> for Gauge {
	fn from_lua(value: Value<'lua>, _: &'lua Lua) -> mlua::Result<Self> {
		match value {
			Value::UserData(ud) => Ok(ud.borrow::<Self>()?.clone()),
			_ => Err(mlua::Error::FromLuaConversionError {
				from:    value.type_name(),
				to:      "Gauge",
				message: Some("expected a Gauge".to_string()),
			}),
		}
	}
}

impl UserData for Gauge {
	fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
		methods.add_function("percent", |_, (ud, percent): (AnyUserData, u8)| {
			if percent > 100 {
				return Err(mlua::Error::RuntimeError("percent must be between 0 and 100".to_string()));
			}

			ud.borrow_mut::<Self>()?.ratio = percent as f64 / 100.0;
			Ok(ud)
		});

		methods.add_function("ratio", |_, (ud, ratio): (AnyUserData, f64)| {
			if !(0.0..1.0).contains(&ratio) {
				return Err(mlua::Error::RuntimeError("ratio must be between 0 and 1".to_string()));
			}

			ud.borrow_mut::<Self>()?.ratio = ratio;
			Ok(ud)
		});

		methods.add_function("label", |_, (ud, label): (AnyUserData, Span)| {
			ud.borrow_mut::<Self>()?.label = Some(label.0);
			Ok(ud)
		});

		methods.add_function("style", |_, (ud, value): (AnyUserData, Value)| {
			ud.borrow_mut::<Self>()?.style = match value {
				Value::Nil => None,
				Value::Table(tbl) => Some(Style::from(tbl).0),
				Value::UserData(ud) => Some(ud.borrow::<Style>()?.0),
				_ => return Err(mlua::Error::external("expected a Style or Table or nil")),
			};
			Ok(ud)
		});

		methods.add_function("gauge_style", |_, (ud, value): (AnyUserData, Value)| {
			ud.borrow_mut::<Self>()?.gauge_style = match value {
				Value::Nil => None,
				Value::Table(tbl) => Some(Style::from(tbl).0),
				Value::UserData(ud) => Some(ud.borrow::<Style>()?.0),
				_ => return Err(mlua::Error::external("expected a Style or Table or nil")),
			};
			Ok(ud)
		});
	}
}
