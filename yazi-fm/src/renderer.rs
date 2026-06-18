use mlua::{ObjectLike, Table};
use ratatui_core::{buffer::Buffer, layout::Rect};
use yazi_core::Core;
use yazi_plugin::LUA;
use yazi_widgets::{Renderable, Renderables};

pub(super) struct Renderer<'a> {
	core:        &'a mut Core,
	component:   &'a str,
	constructor: &'a str,
	redrawer:    &'a str,
}

impl<'a> Renderer<'a> {
	pub(super) fn new(core: &'a mut Core, component: &'a str) -> Self {
		Self { core, component, constructor: "new", redrawer: "redraw" }
	}

	pub(super) fn with_constructor(mut self, constructor: &'a str) -> Self {
		self.constructor = constructor;
		self
	}

	pub(super) fn with_redrawer(mut self, redrawer: &'a str) -> Self {
		self.redrawer = redrawer;
		self
	}

	pub(super) fn render(self, area: Rect, buf: &mut Buffer) -> mlua::Result<()> {
		let area = yazi_binding::elements::Rect::from(area);

		let value = LUA
			.globals()
			.raw_get::<Table>(self.component)?
			.call_method::<Table>(self.constructor, area)?
			.call_method(self.redrawer, ())?;

		self.core.input.alt = None;
		Renderables::reduce(value, |element| {
			match &element {
				Renderable::Input(input) if input.focus => {
					self.core.input.alt = Some(input.into());
				}
				_ => {}
			}

			element.render_with(buf, |p| self.core.mgr.area(p));
		});

		Ok::<_, mlua::Error>(())
	}
}
