use anyhow::Result;
use yazi_macro::{act, render};
use yazi_parser::VoidOpt;
use yazi_shared::data::Data;

use crate::input::Input;

impl Input {
	pub fn redo(&mut self, _: VoidOpt) -> Result<Data> {
		render!(self.snaps.redo());

		act!(r#move, self)
	}
}
