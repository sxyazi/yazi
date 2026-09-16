use anyhow::Result;
use yazi_macro::render;
use yazi_shared::data::Data;

use crate::input::Input;

impl Input {
	pub(crate) fn redo(&mut self, _: ()) -> Result<Data> {
		render!(self.snaps.redo());

		act!(r#move, self)
	}
}
