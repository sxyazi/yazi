use yazi_shared::{event::{Cmd, Data}, render};

use crate::tasks::Tasks;

pub struct Opt {
	step: isize,
}

impl From<Cmd> for Opt {
	fn from(c: Cmd) -> Self { Self { step: c.first().and_then(Data::as_isize).unwrap_or(0) } }
}

impl From<isize> for Opt {
	fn from(step: isize) -> Self { Self { step } }
}

impl Tasks {
	pub fn arrow(&mut self, opt: impl Into<Opt>) {
		let old = self.cursor;
		if opt.into().step > 0 {
			self.cursor += 1;
		} else {
			self.cursor = self.cursor.saturating_sub(1);
		}

		let max = Self::limit().min(self.summaries.len());
		self.cursor = self.cursor.min(max.saturating_sub(1));
		render!(self.cursor != old);
	}
}
