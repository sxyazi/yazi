use yazi_shared::{event::{Cmd, Data}, fs::Url};

use crate::{manager::Manager, tasks::Tasks};

#[derive(Default)]
pub struct Opt {
	page:    Option<usize>,
	only_if: Option<Url>,
}

impl From<Cmd> for Opt {
	fn from(mut c: Cmd) -> Self {
		Self {
			page:    c.first().and_then(Data::as_usize),
			only_if: c.take("only-if").and_then(Data::into_url),
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

		if opt.only_if.is_some_and(|u| u != *self.active().cwd()) {
			return;
		}

		let targets = self.current().paginate(opt.page.unwrap_or(self.current().page));
		tasks.fetch_paged(targets, &self.mimetype);
		tasks.preload_paged(targets, &self.mimetype);
	}
}
