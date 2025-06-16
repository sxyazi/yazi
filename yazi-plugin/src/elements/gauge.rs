use mlua::{AnyUserData, ExternalError, IntoLua, Lua, MetaMethod, Table, UserData, UserDataMethods, Value};
use ratatui::widgets::Widget;
use yazi_binding::Style;

use super::{Area, Span};

#[derive(Clone, Debug, Default)]
pub struct Gauge {
	area: Area,

	ratio:       f64,
	label:       Option<ratatui::text::Span<'static>>,
	style:       ratatui::style::Style,
	gauge_style: ratatui::style::Style,
}

impl Gauge {
	pub fn compose(lua: &Lua) -> mlua::Result<Value> {
		let new = lua.create_function(|_, _: Table| Ok(Gauge::default()))?;

		let gauge = lua.create_table()?;
		gauge.set_metatable(Some(lua.create_table_from([(MetaMethod::Call.name(), new)])?));

		gauge.into_lua(lua)
	}

	pub(super) fn render(
		self,
		buf: &mut ratatui::buffer::Buffer,
		trans: impl FnOnce(yazi_config::popup::Position) -> ratatui::layout::Rect,
	) {
		let mut gauge = ratatui::widgets::Gauge::default()
			.ratio(self.ratio)
			.style(self.style)
			.gauge_style(self.gauge_style);

		if let Some(s) = self.label {
			gauge = gauge.label(s)
		}

		gauge.render(self.area.transform(trans), buf);
	}
}

impl UserData for Gauge {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		crate::impl_area_method!(methods);
		crate::impl_style_method!(methods, style);

		methods.add_function_mut("percent", |_, (ud, percent): (AnyUserData, u8)| {
			if percent > 100 {
				return Err("percent must be between 0 and 100".into_lua_err());
			}

			ud.borrow_mut::<Self>()?.ratio = percent as f64 / 100.0;
			Ok(ud)
		});

		methods.add_function_mut("ratio", |_, (ud, ratio): (AnyUserData, f64)| {
			if !(0.0..1.0).contains(&ratio) {
				return Err("ratio must be between 0 and 1".into_lua_err());
			}

			ud.borrow_mut::<Self>()?.ratio = ratio;
			Ok(ud)
		});

		methods.add_function_mut("label", |_, (ud, label): (AnyUserData, Value)| {
			ud.borrow_mut::<Self>()?.label = Some(Span::try_from(label)?.0);
			Ok(ud)
		});

		methods.add_function_mut("gauge_style", |_, (ud, value): (AnyUserData, Value)| {
			ud.borrow_mut::<Self>()?.gauge_style = Style::try_from(value)?.0;
			Ok(ud)
		});
	}
}
