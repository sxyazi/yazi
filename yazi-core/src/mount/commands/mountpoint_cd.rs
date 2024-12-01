use yazi_macro::emit;
use yazi_proxy::options::ProcessExecOpt;
use yazi_shared::{Layer, event::Cmd, fs::Url};

use crate::mount::Mount;

impl Mount {
	pub fn mountpoint_cd(&mut self, _opt: impl TryInto<ProcessExecOpt>) {
		if let Some(target) = self.points.get(self.cursor) {
			let url: Url = target.path.clone().into();
			emit!(Call(Cmd::args("cd", &[url]), Layer::Manager));
		}
	}
}
