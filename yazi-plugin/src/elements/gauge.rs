use mlua::{AnyUserData, ExternalError, Lua, Table, UserData, UserDataMethods, Value};
use ratatui::widgets::Widget;

use super::{RectRef, Renderable, Span};
use crate::elements::Style;

#[derive(Clone, Default)]
pub struct Gauge {
	area: ratatui::layout::Rect,

	ratio:       f64,
	label:       Option<ratatui::text::Span<'static>>,
	style:       ratatui::style::Style,
	gauge_style: ratatui::style::Style,
}

impl Gauge {
	pub fn install(lua: &Lua, ui: &Table) -> mlua::Result<()> {
		ui.raw_set(
			"Gauge",
			lua.create_function(|_, area: RectRef| Ok(Gauge { area: *area, ..Default::default() }))?,
		)
	}
}

impl UserData for Gauge {
	fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
		crate::impl_style_method!(methods, style);

		methods.add_function("percent", |_, (ud, percent): (AnyUserData, u8)| {
			if percent > 100 {
				return Err("percent must be between 0 and 100".into_lua_err());
			}

			ud.borrow_mut::<Self>()?.ratio = percent as f64 / 100.0;
			Ok(ud)
		});

		methods.add_function("ratio", |_, (ud, ratio): (AnyUserData, f64)| {
			if !(0.0..1.0).contains(&ratio) {
				return Err("ratio must be between 0 and 1".into_lua_err());
			}

			ud.borrow_mut::<Self>()?.ratio = ratio;
			Ok(ud)
		});

		methods.add_function("label", |_, (ud, label): (AnyUserData, Span)| {
			ud.borrow_mut::<Self>()?.label = Some(label.0);
			Ok(ud)
		});

		methods.add_function("gauge_style", |_, (ud, value): (AnyUserData, Value)| {
			ud.borrow_mut::<Self>()?.gauge_style = Style::try_from(value)?.0;
			Ok(ud)
		});
	}
}

impl Renderable for Gauge {
	fn area(&self) -> ratatui::layout::Rect { self.area }

	fn render(self: Box<Self>, buf: &mut ratatui::buffer::Buffer) {
		let mut gauge = ratatui::widgets::Gauge::default()
			.ratio(self.ratio)
			.style(self.style)
			.gauge_style(self.gauge_style);

		if let Some(s) = self.label {
			gauge = gauge.label(s)
		}

		gauge.render(self.area, buf);
	}

	fn clone_render(&self, buf: &mut ratatui::buffer::Buffer) { Box::new(self.clone()).render(buf) }
}
