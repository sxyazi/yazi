use yazi_plugin::utils::PreviewLock;
use yazi_shared::event::Cmd;

use crate::tab::Tab;

pub struct Opt {
	lock: PreviewLock,
}

impl TryFrom<Cmd> for Opt {
	type Error = ();

	fn try_from(mut c: Cmd) -> Result<Self, Self::Error> {
		Ok(Self { lock: c.take_any("lock").ok_or(())? })
	}
}

impl Tab {
	pub fn update_spotted(&mut self, opt: impl TryInto<Opt>) {
		todo!();
	}
}
