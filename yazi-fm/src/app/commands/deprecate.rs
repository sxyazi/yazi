use anyhow::Result;
use yazi_macro::succ;
use yazi_parser::app::{DeprecateOpt, NotifyLevel, NotifyOpt};
use yazi_shared::event::Data;

use crate::app::App;

impl App {
	pub(crate) fn deprecate(&mut self, opt: DeprecateOpt) -> Result<Data> {
		succ!(self.core.notify.push(NotifyOpt {
			title:   "Deprecated API".to_owned(),
			content: opt.content,
			level:   NotifyLevel::Warn,
			timeout: std::time::Duration::from_secs(20),
		}));
	}
}
