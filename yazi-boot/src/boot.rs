use std::ops::Deref;

use yazi_fs::CWD;
use yazi_shared::{id::Id, url::UrlBuf};
use yazi_shim::cell::RoCell;

use crate::Args;

pub static BOOT: RoCell<Boot> = RoCell::new();

#[derive(Debug)]
pub struct Boot {
	args:    Args,
	pub id:  Id,
	pub cwd: UrlBuf,
}

impl Deref for Boot {
	type Target = Args;

	fn deref(&self) -> &Self::Target { &self.args }
}

impl Default for Boot {
	fn default() -> Self { Args::default().into() }
}

impl From<Args> for Boot {
	fn from(args: Args) -> Self {
		Self { id: args.client_id.unwrap_or_else(Id::unique), cwd: CWD.load().as_ref().clone(), args }
	}
}
