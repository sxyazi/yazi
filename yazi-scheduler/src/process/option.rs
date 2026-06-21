use std::ffi::OsString;

use yazi_macro::impl_data_any;
use yazi_shared::url::{UrlBuf, UrlCow};

// TODO: remove in favor of ShellForm
#[derive(Clone, Debug)]
pub struct ProcessOpt {
	pub cwd:    UrlBuf,
	pub cmd:    OsString,
	pub args:   Vec<UrlCow<'static>>,
	pub block:  bool,
	pub orphan: bool,

	pub spread: bool, // TODO: remove
}

impl_data_any!(ProcessOpt);
