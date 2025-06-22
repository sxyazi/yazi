use yazi_fs::Step;

pub trait Scrollable {
	fn len(&self) -> usize;
	fn limit(&self) -> usize;
	fn scrolloff(&self) -> usize { self.limit() / 2 }
	fn cursor_mut(&mut self) -> &mut usize;
	fn offset_mut(&mut self) -> &mut usize;

	fn scroll(&mut self, step: Step) -> bool {
		let new = step.add(*self.cursor_mut(), self.len(), self.limit());
		if new > *self.cursor_mut() { self.next(new) } else { self.prev(new) }
	}

	fn next(&mut self, n_cur: usize) -> bool {
		let (o_cur, o_off) = (*self.cursor_mut(), *self.offset_mut());
		let (len, limit, scrolloff) = (self.len(), self.limit(), self.scrolloff());

		let n_off = if n_cur < len.min(o_off + limit).saturating_sub(scrolloff) {
			o_off.min(len.saturating_sub(1))
		} else {
			len.saturating_sub(limit).min(o_off + n_cur - o_cur)
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
			self.len().saturating_sub(1).min(o_off)
		};

		*self.cursor_mut() = n_cur;
		*self.offset_mut() = n_off;
		(n_cur, n_off) != (o_cur, o_off)
	}
}
