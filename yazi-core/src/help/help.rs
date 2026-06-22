use anyhow::Result;
use ratatui_widgets::block::Padding;
use yazi_binding::position::Position;
use yazi_config::{KEYMAP, keymap::ChordArc};
use yazi_macro::render;
use yazi_shared::Layer;
use yazi_term::event::KeyEvent;
use yazi_tty::sequence::SetCursorStyle;
use yazi_widgets::{Scrollable, input::Input};

#[derive(Default)]
pub struct Help {
	pub visible:  bool,
	pub layer:    Layer,
	pub position: Position,
	pub bindings: Vec<ChordArc>,

	// Filter
	pub input:   Input,
	pub keyword: String,

	pub offset: usize,
	pub cursor: usize,
	pub height: u16,
}

impl Help {
	pub fn r#type(&mut self, key: KeyEvent) -> Result<bool> {
		if !self.input.r#type(key)? {
			return Ok(false);
		}

		self.filter_apply();
		Ok(true)
	}

	pub fn filter_apply(&mut self) {
		let kw = self.input.value();

		if kw.is_empty() {
			self.keyword.clear();
			self.bindings = KEYMAP.chords(self.layer).iter().cloned().collect();
		} else if self.keyword != kw {
			let lowercased = kw.to_lowercase();
			self.keyword = kw.to_owned();
			self.bindings = KEYMAP
				.chords(self.layer)
				.iter()
				.filter(|&c| c.desc_or_run().to_lowercase().contains(&lowercased))
				.cloned()
				.collect();
		}

		render!(self.scroll(0));
	}
}

impl Help {
	pub fn padding(&self) -> Padding { Padding::new(1, 1, 1, 1) }

	// --- Bindings
	pub fn window(&self) -> &[ChordArc] {
		let end = (self.offset + self.limit()).min(self.bindings.len());
		&self.bindings[self.offset..end]
	}

	// --- Cursor
	pub fn cursor(&self) -> Option<u16> { self.visible.then_some(self.input.cursor()) }

	pub fn rel_cursor(&self) -> usize { self.cursor - self.offset }

	pub fn cursor_shape(&self) -> Option<SetCursorStyle> {
		self.visible.then_some(self.input.cursor_shape())
	}
}

impl Scrollable for Help {
	fn total(&self) -> usize { self.bindings.len() }

	fn limit(&self) -> usize {
		let p = self.padding();
		self.height.saturating_sub(p.top + /* input */ 1 + /* divider */ 1 + p.bottom) as usize
	}

	fn cursor_mut(&mut self) -> &mut usize { &mut self.cursor }

	fn offset_mut(&mut self) -> &mut usize { &mut self.offset }
}
