use anyhow::Result;
use yazi_macro::{act, succ};
use yazi_shared::{data::Data, event::ActionCow};

use crate::input::{Input, InputMode};

impl Input {
	pub fn execute(&mut self, action: ActionCow) -> Result<Data> {
		macro_rules! on {
			($name:ident) => {
				if action.name == stringify!($name) {
					return act!($name, self, action);
				}
			};
			($name:ident, $alias:literal) => {
				if action.name == $alias {
					return act!($name, self, action);
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
