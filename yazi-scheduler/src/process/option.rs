use std::ffi::OsString;

use yazi_shared::url::UrlCow;

// TODO: remove in favor of ShellForm
#[derive(Clone, Debug)]
pub struct ProcessOpt {
	pub cwd:    UrlCow<'static>,
	pub cmd:    OsString,
	pub args:   Vec<UrlCow<'static>>,
	pub block:  bool,
	pub orphan: bool,

	pub spread: bool, // TODO: remove
}
