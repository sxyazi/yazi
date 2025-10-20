use std::ops::Range;

use anyhow::Result;
use yazi_macro::{act, render, succ};
use yazi_parser::input::CasefyOpt;
use yazi_shared::data::Data;

use crate::input::{Input, op::InputOp};

impl Input {
	pub fn casefy(&mut self, opt: CasefyOpt) -> Result<Data> {
		let snap = self.snap_mut();
		if !matches!(snap.op, InputOp::Select(_)) {
			succ!();
		}

		let range = snap.op.range(snap.cursor, true).unwrap();
		let Range { start, end } = snap.idx(range.start)..snap.idx(range.end);

		let (start, end) = (start.unwrap(), end.unwrap());
		let casefied = opt.transform(&snap.value[start..end]);

		snap.value.replace_range(start..end, &casefied);
		snap.op = InputOp::None;
		snap.cursor = range.start;
		self.snaps.tag(self.limit).then(|| self.flush_value());

		act!(r#move, self)?;
		succ!(render!());
	}
}
