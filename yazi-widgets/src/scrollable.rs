use crate::Step;

pub trait Scrollable {
	fn total(&self) -> usize;
	fn limit(&self) -> usize;
	fn scrolloff(&self) -> usize { self.limit() / 2 }
	fn cursor_mut(&mut self) -> &mut usize;
	fn offset_mut(&mut self) -> &mut usize;

	/// End of the visible window (exclusive)
	fn visible_end(&mut self) -> usize { self.total().min(*self.offset_mut() + self.limit()) }

	/// Cursor is past the bottom safe zone
	fn needs_scroll_down(&mut self, cursor: usize) -> bool {
		cursor >= self.visible_end().saturating_sub(self.scrolloff())
	}

	/// Cursor is in the top scrolloff zone
	fn needs_scroll_up(&mut self, cursor: usize) -> bool {
		cursor < *self.offset_mut() + self.scrolloff()
	}

	fn scroll(&mut self, step: impl Into<Step>) -> bool {
		let old = *self.cursor_mut();
		let new =
			step.into().add(old, self.total(), self.limit(), *self.offset_mut(), self.scrolloff());

		if new > old { self.next(new) } else { self.prev(new) }
	}

	fn next(&mut self, n_cur: usize) -> bool {
		let (o_cur, o_off) = (*self.cursor_mut(), *self.offset_mut());
		let (total, limit) = (self.total(), self.limit());

		let n_off = if self.needs_scroll_down(n_cur) {
			total.saturating_sub(limit).min(o_off + n_cur - o_cur)
		} else {
			o_off.min(total.saturating_sub(1))
		};

		*self.cursor_mut() = n_cur;
		*self.offset_mut() = n_off;
		(n_cur, n_off) != (o_cur, o_off)
	}

	fn prev(&mut self, n_cur: usize) -> bool {
		let (o_cur, o_off) = (*self.cursor_mut(), *self.offset_mut());

		let n_off = if self.needs_scroll_up(n_cur) {
			o_off.saturating_sub(o_cur - n_cur)
		} else {
			self.total().saturating_sub(1).min(o_off)
		};

		*self.cursor_mut() = n_cur;
		*self.offset_mut() = n_off;
		(n_cur, n_off) != (o_cur, o_off)
	}
}
