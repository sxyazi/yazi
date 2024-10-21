use yazi_shared::event::{Cmd, Data};

use crate::spot::Spot;

struct Opt {
	step: isize,
}

impl From<Cmd> for Opt {
	fn from(c: Cmd) -> Self { Self { step: c.first().and_then(Data::as_isize).unwrap_or(0) } }
}

impl Spot {
	#[yazi_codegen::command]
	pub fn arrow(&mut self, _: Opt) {
		todo!();
	}
}
