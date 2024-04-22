use std::collections::HashSet;

use yazi_shared::{event::Cmd, fs::Url, render};

use crate::manager::{Manager, Yanked};

#[derive(Default)]
pub struct Opt {
	cut:  bool,
	urls: HashSet<Url>,
}

impl TryFrom<Cmd> for Opt {
	type Error = ();

	fn try_from(mut c: Cmd) -> Result<Self, Self::Error> {
		if let Some(iter) = c.take_any::<yazi_dds::body::BodyYankIter>("urls") {
			Ok(Self { urls: iter.urls.into_iter().collect(), cut: iter.cut })
		} else {
			Err(())
		}
	}
}

impl Manager {
	pub fn update_yanked(&mut self, opt: impl TryInto<Opt>) {
		let Ok(opt) = opt.try_into() else { return };

		if opt.urls.is_empty() && self.yanked.is_empty() {
			return;
		}

		self.yanked = Yanked::new(opt.cut, opt.urls);
		render!();
	}
}
