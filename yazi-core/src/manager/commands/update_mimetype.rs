use std::collections::HashMap;

use yazi_plugin::ValueSendable;
use yazi_shared::{event::Exec, fs::Url};

use crate::{manager::Manager, tasks::Tasks};

pub struct Opt {
	data: ValueSendable,
}

impl TryFrom<&Exec> for Opt {
	type Error = ();

	fn try_from(e: &Exec) -> Result<Self, Self::Error> { Ok(Self { data: e.take_data().ok_or(())? }) }
}

impl Manager {
	pub fn update_mimetype(&mut self, opt: impl TryInto<Opt>, tasks: &Tasks) -> bool {
		let Ok(opt) = opt.try_into() else {
			return false;
		};

		let updates: HashMap<_, _> = opt
			.data
			.into_table_string()
			.into_iter()
			.map(|(url, mime)| (Url::from(url), mime))
			.filter(|(url, mime)| self.mimetype.get(url) != Some(mime))
			.collect();

		if updates.is_empty() {
			return false;
		}

		let paged = self.current().paginate(self.current().page);
		tasks.preload_affected(paged, &updates);

		self.mimetype.extend(updates);
		self.peek(false);
		true
	}
}
