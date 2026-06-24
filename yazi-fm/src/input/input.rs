use std::path::Path;

use ratatui_core::{buffer::Buffer, layout::{Margin, Rect}, text::Line, widgets::Widget};
use ratatui_widgets::{block::Block, borders::BorderType};
use yazi_config::{Icon, THEME};
use yazi_core::Core;
use yazi_fs::{cha::{Cha, ChaKind}, file::File};
use yazi_shim::path::CROSS_SEPARATOR;

pub(crate) struct Input<'a> {
	core: &'a Core,
}

impl<'a> Input<'a> {
	pub(crate) fn new(core: &'a Core) -> Self { Self { core } }

	fn icon(&self) -> Option<Icon> {
		let input = &self.core.input.main;

		let is_dir = input.value().ends_with(CROSS_SEPARATOR);
		let is_hovered = input.id.starts_with("rename-");

		let (path, mode): (&str, u16) = match input.id.as_str() {
			"cd" => (input.value(), if is_dir { 0o40755 } else { 0o100644 }),
			"create" => (input.value(), if is_dir { 0o40755 } else { 0o100644 }),
			"rename-file" => (input.value(), 0o100644),
			"rename-dir" => (input.value(), 0o40755),
			"shell" => ("icon.sh", 0o100644),
			_ => return None,
		};

		THEME.icon.matches(
			&File {
				url:     Path::new(path).into(),
				cha:     Cha { kind: ChaKind::empty(), mode: mode.try_into().ok()?, ..Default::default() },
				link_to: None,
			},
			is_hovered,
		)
	}
}

impl Widget for Input<'_> {
	fn render(self, _: Rect, buf: &mut Buffer) {
		let input = &self.core.input.main;

		let outer = self.core.mgr.area(input.position);
		yazi_widgets::clear::Clear::default().render(outer, buf);

		let mut block = Block::bordered()
			.border_type(BorderType::Rounded)
			.border_style(THEME.input.border.get())
			.title(Line::styled(&input.title, THEME.input.title.get()));

		if let Some(i) = self.icon() {
			block = block.title_bottom(Line::raw(format!("{} ", i.text)).style(i.style).right_aligned());
		}

		block.render(outer, buf);
		input.render(outer.inner(Margin::new(1, 1)), buf);
	}
}
