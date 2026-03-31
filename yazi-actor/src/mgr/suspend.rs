use anyhow::Result;
use yazi_macro::succ;
use yazi_parser::VoidForm;
use yazi_shared::data::Data;

use crate::{Actor, Ctx};

pub struct Suspend;

impl Actor for Suspend {
	type Form = VoidForm;

	const NAME: &str = "suspend";

	fn act(_: &mut Ctx, _: Self::Form) -> Result<Data> {
		#[cfg(unix)]
		if !yazi_shared::session_leader() {
			unsafe {
				libc::raise(libc::SIGTSTP);
			}
		}
		succ!();
	}
}
