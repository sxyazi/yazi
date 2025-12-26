use anyhow::Result;
use yazi_macro::{act, succ};
use yazi_shared::{data::Data, event::CmdCow};

use crate::input::{Input, InputMode};

impl Input {
	pub fn execute(&mut self, cmd: CmdCow) -> Result<Data> {
		macro_rules! on {
			($name:ident) => {
				if cmd.name == stringify!($name) {
					return act!($name, self, cmd);
				}
			};
			($name:ident, $alias:literal) => {
				if cmd.name == $alias {
					return act!($name, self, cmd);
				}
			};
		}

		on!(r#move, "move");
		on!(backward);
		on!(forward);

		match self.mode() {
			InputMode::Normal => {
				on!(insert);
				on!(visual);
				on!(replace);

				on!(delete);
				on!(yank);
				on!(paste);

				on!(undo);
				on!(redo);

				on!(casefy);
			}
			InputMode::Insert => {
				on!(visual);

				on!(backspace);
				on!(kill);
			}
			InputMode::Replace => {}
		}

		succ!();
	}
}
