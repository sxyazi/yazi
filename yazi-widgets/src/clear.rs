use std::sync::atomic::{AtomicBool, Ordering};

use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
use unicode_width::UnicodeWidthStr;
use yazi_adapter::ADAPTOR;

pub static COLLISION: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Copy, Debug)]
pub struct Clear;

impl Widget for Clear {
	fn render(self, area: Rect, buf: &mut Buffer)
	where
		Self: Sized,
	{
		// Patch cells straddling the area's left/right edge before clearing.
		//
		// On the left edge, a double-width glyph in the layer below sitting
		// at `area.x - 1` has its right half at `area.x`, *inside* the area.
		// `ratatui::widgets::Clear` only resets cells inside the area, so
		// after the clear the back-layer cell at `area.x - 1` would still
		// claim two cells of visual width and overlap whatever the caller
		// draws on top — the bug pattern in #3947.
		//
		// The mirrored case at the right edge: a double-width glyph at
		// `area.right() - 1` has its left half inside the area (cleared
		// later) and its continuation at `area.right()`, *outside* the
		// area. The continuation cell is left as an empty string by
		// ratatui's buffer model, so the terminal would either render
		// stale content from a previous frame in that column or leave the
		// neighbour glyph half-rendered. Blank the continuation here, while
		// we still know it was a continuation, before the inside cell is
		// cleared.
		if area.x > 0 {
			for y in area.top()..area.bottom() {
				let cell = &mut buf[(area.x - 1, y)];
				if cell.symbol().width() > 1 {
					cell.set_symbol(" ");
				}
			}
		}
		if area.right() < buf.area().right() {
			for y in area.top()..area.bottom() {
				if buf[(area.right() - 1, y)].symbol().width() > 1 {
					buf[(area.right(), y)].set_symbol(" ");
				}
			}
		}

		ratatui::widgets::Clear.render(area, buf);

		let Some(r) = ADAPTOR.get().shown_load().and_then(|r| overlap(area, r)) else {
			return;
		};

		ADAPTOR.get().image_erase(r).ok();
		COLLISION.store(true, Ordering::Relaxed);
		for y in r.top()..r.bottom() {
			for x in r.left()..r.right() {
				buf[(x, y)].set_skip(true);
			}
		}
	}
}

const fn is_overlapping(a: Rect, b: Rect) -> bool {
	a.x < b.right() && a.right() > b.x && a.y < b.y + b.height && a.y + a.height > b.y
}

fn overlap(a: Rect, b: Rect) -> Option<Rect> {
	if !is_overlapping(a, b) {
		return None;
	}

	let x = a.x.max(b.x);
	let y = a.y.max(b.y);
	let width = a.right().min(b.right()) - x;
	let height = (a.y + a.height).min(b.y + b.height) - y;
	Some(Rect { x, y, width, height })
}

#[cfg(test)]
mod tests {
	use ratatui::style::Style;

	use super::*;

	// Fill `buf` with `s` at row 0 and run our `Clear` over `area`.
	// Returns the row 0 symbols in order, including the empty continuation
	// strings that ratatui leaves for the right halves of double-width
	// glyphs. This makes the asymmetry between "single-width spaces" and
	// "stale continuation markers" visible in the assertions.
	fn run(s: &str, buf_w: u16, area: Rect) -> Vec<String> {
		let mut buf = Buffer::empty(Rect::new(0, 0, buf_w, 1));
		buf.set_string(0, 0, s, Style::default());
		Clear.render(area, &mut buf);
		(0..buf_w).map(|x| buf[(x, 0)].symbol().to_string()).collect()
	}

	#[test]
	fn left_edge_double_width_replaced_with_space() {
		// Back layer:  a b c 字 字
		//              0 1 2 3 4   <- column
		// Overlay starts at column 4 (right half of 字). Pre-fix, cell 3
		// keeps "字" and overlaps the overlay's left edge.
		let row = run("abc字", 6, Rect::new(4, 0, 2, 1));
		assert_eq!(row[3], " ", "double-width at the left edge should be blanked");
		// Cells inside the area are cleared by ratatui::widgets::Clear.
		assert_eq!(row[4], " ");
		assert_eq!(row[5], " ");
	}

	#[test]
	fn left_edge_single_width_left_alone() {
		// Cell 3 is a plain ASCII char. We must NOT blank it.
		let row = run("abcde", 6, Rect::new(4, 0, 2, 1));
		assert_eq!(row[3], "d");
	}

	#[test]
	fn right_edge_continuation_cleared() {
		// Back layer:  a b c 字 d
		//              0 1 2 3 4 5   (字 occupies cols 3 and 4)
		// Overlay covers columns 0..4. The left half of 字 is the last
		// cell *inside* the area and gets reset to " " by the inner clear.
		// The right half at column 4 is the continuation cell, marked with
		// an empty symbol by ratatui's buffer model and *outside* the area
		// — pre-fix it stays empty and the terminal renders stale content
		// there; post-fix it's blanked to " ".
		let row = run("abc字d", 6, Rect::new(0, 0, 4, 1));
		assert_eq!(row[3], " ");
		assert_eq!(row[4], " ", "stranded continuation should be blanked");
		assert_eq!(row[5], "d", "neighbour past the continuation must be left alone");
	}

	#[test]
	fn right_edge_with_no_overhang_leaves_neighbour_alone() {
		// Cell at area.right() - 1 is single-width, so cell at area.right()
		// is a normal back-layer cell and must not be overwritten.
		let row = run("abcde", 6, Rect::new(0, 0, 3, 1));
		assert_eq!(row[3], "d");
	}

	#[test]
	fn area_at_buffer_edges_does_not_panic() {
		// area.x == 0 and area.right() == buf_w must not be touched (the
		// boundary cells we'd patch don't exist).
		let row = run("abcd", 4, Rect::new(0, 0, 4, 1));
		assert_eq!(row, vec![" ", " ", " ", " "]);
	}
}
