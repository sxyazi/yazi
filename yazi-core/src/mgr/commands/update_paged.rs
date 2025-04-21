use yazi_shared::{event::{CmdCow, Data}, url::Url};

use crate::{mgr::Mgr, tasks::Tasks};

#[derive(Default)]
pub struct Opt {
	page:    Option<usize>,
	only_if: Option<Url>,
}

impl From<CmdCow> for Opt {
	fn from(mut c: CmdCow) -> Self {
		Self { page: c.first().and_then(Data::as_usize), only_if: c.take_url("only-if") }
	}
}

impl From<()> for Opt {
	fn from(_: ()) -> Self { Self::default() }
}

impl Mgr {
	pub fn update_paged(&mut self, opt: impl TryInto<Opt>, tasks: &Tasks) {
		let Ok(opt): Result<Opt, _> = opt.try_into() else {
			return;
		};

		if opt.only_if.is_some_and(|u| u != *self.cwd()) {
			return;
		}

		let targets = self.current().paginate(opt.page.unwrap_or(self.current().page));
		if !targets.is_empty() {
			tasks.fetch_paged(targets, &self.mimetype);
			tasks.preload_paged(targets, &self.mimetype);
		}
	}
}
