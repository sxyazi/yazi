use yazi_config::keymap::Exec;

use crate::tasks::Tasks;

pub struct Opt;

impl From<&Exec> for Opt {
	fn from(_: &Exec) -> Self { Self }
}
impl From<()> for Opt {
	fn from(_: ()) -> Self { Self }
}

impl Tasks {
	pub fn toggle(&mut self, _: impl Into<Opt>) -> bool {
		self.visible = !self.visible;
		true
	}
}
