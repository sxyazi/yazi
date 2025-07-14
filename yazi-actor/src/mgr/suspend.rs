use anyhow::Result;
use yazi_macro::succ;
use yazi_parser::VoidOpt;
use yazi_shared::event::Data;

use crate::{Actor, Ctx};

pub struct Suspend;

impl Actor for Suspend {
	type Options = VoidOpt;

	const NAME: &'static str = "suspend";

	fn act(_: &mut Ctx, _: Self::Options) -> Result<Data> {
		#[cfg(unix)]
		unsafe {
			libc::raise(libc::SIGTSTP);
		}
		succ!();
	}
}
