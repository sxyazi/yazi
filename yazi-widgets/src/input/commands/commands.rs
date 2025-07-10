use yazi_shared::event::CmdCow;

use crate::input::{Input, InputMode};

impl Input {
	pub fn execute(&mut self, cmd: CmdCow) {
		macro_rules! on {
			($name:ident) => {
				if cmd.name == stringify!($name) {
					return self.$name(cmd);
				}
			};
			($name:ident, $alias:literal) => {
				if cmd.name == $alias {
					return self.$name(cmd);
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
			}
			InputMode::Insert => {
				on!(visual);

				on!(backspace);
				on!(kill);

				on!(complete);
			}
			InputMode::Replace => {}
		}
	}
}
