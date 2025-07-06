use yazi_parser::spot::CloseOpt;

use crate::spot::Spot;

impl Spot {
	#[yazi_codegen::command]
	pub fn close(&mut self, _: CloseOpt) { self.reset(); }
}
