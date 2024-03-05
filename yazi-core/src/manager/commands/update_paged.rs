use yazi_shared::{event::Cmd, fs::Url};

use crate::{manager::Manager, tasks::Tasks};

#[derive(Default)]
pub struct Opt {
	page:    Option<usize>,
	only_if: Option<Url>,
}

impl From<Cmd> for Opt {
	fn from(mut c: Cmd) -> Self {
		Self {
			page:    c.take_first().and_then(|s| s.parse().ok()),
			only_if: c.take_name("only-if").map(Url::from),
		}
	}
}

impl From<()> for Opt {
	fn from(_: ()) -> Self { Self::default() }
}

impl Manager {
	pub fn update_paged(&mut self, opt: impl TryInto<Opt>, tasks: &Tasks) {
		let Ok(opt) = opt.try_into() else {
			return;
		};

		if opt.only_if.is_some_and(|u| u != self.current().cwd) {
			return;
		}

		let targets = self.current().paginate(opt.page.unwrap_or(self.current().page));
		tasks.preload_paged(targets, &self.mimetype);
	}
}
