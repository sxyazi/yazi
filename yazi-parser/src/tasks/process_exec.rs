use std::{borrow::Cow, ffi::OsString};

use anyhow::anyhow;
use tokio::sync::oneshot;
use yazi_config::opener::OpenerRule;
use yazi_shared::{event::CmdCow, url::Url};

// --- Exec
pub struct ProcessExecOpt {
	pub cwd:    Url,
	pub opener: Cow<'static, OpenerRule>,
	pub args:   Vec<OsString>,
	pub done:   Option<oneshot::Sender<()>>,
}

impl TryFrom<CmdCow> for ProcessExecOpt {
	type Error = anyhow::Error;

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> {
		c.take_any("option").ok_or_else(|| anyhow!("Missing 'option' in ProcessExecOpt"))
	}
}
