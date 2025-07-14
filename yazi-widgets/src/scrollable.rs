use yazi_parser::Step;

pub trait Scrollable {
	fn total(&self) -> usize;
	fn limit(&self) -> usize;
	fn scrolloff(&self) -> usize { self.limit() / 2 }
	fn cursor_mut(&mut self) -> &mut usize;
	fn offset_mut(&mut self) -> &mut usize;

	fn scroll(&mut self, step: impl Into<Step>) -> bool {
		let new = step.into().add(*self.cursor_mut(), self.total(), self.limit());
		if new > *self.cursor_mut() { self.next(new) } else { self.prev(new) }
	}

	fn next(&mut self, n_cur: usize) -> bool {
		let (o_cur, o_off) = (*self.cursor_mut(), *self.offset_mut());
		let (total, limit, scrolloff) = (self.total(), self.limit(), self.scrolloff());

		let n_off = if n_cur < total.min(o_off + limit).saturating_sub(scrolloff) {
			o_off.min(total.saturating_sub(1))
		} else {
			total.saturating_sub(limit).min(o_off + n_cur - o_cur)
		};

		*self.cursor_mut() = n_cur;
		*self.offset_mut() = n_off;
		(n_cur, n_off) != (o_cur, o_off)
	}

	fn prev(&mut self, n_cur: usize) -> bool {
		let (o_cur, o_off) = (*self.cursor_mut(), *self.offset_mut());

		let n_off = if n_cur < o_off + self.scrolloff() {
			o_off.saturating_sub(o_cur - n_cur)
		} else {
			self.total().saturating_sub(1).min(o_off)
		};

		*self.cursor_mut() = n_cur;
		*self.offset_mut() = n_off;
		(n_cur, n_off) != (o_cur, o_off)
	}
}
