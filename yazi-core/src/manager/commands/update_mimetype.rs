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

		let linked = self.watcher.linked.read();
		let updates = opt
			.data
			.into_table_string()
			.into_iter()
			.map(|(url, mime)| (Url::from(url), mime))
			.filter(|(url, mime)| self.mimetype.get(url) != Some(mime))
			.fold(HashMap::new(), |mut map, (u, m)| {
				for u in linked.from_file(&u) {
					map.insert(u, m.clone());
				}
				map.insert(u, m);
				map
			});

		drop(linked);
		if updates.is_empty() {
			return false;
		}

		let affected: Vec<_> = self
			.current()
			.paginate(self.current().page)
			.iter()
			.filter(|&f| updates.contains_key(&f.url))
			.cloned()
			.collect();

		self.mimetype.extend(updates);
		self.peek(false);

		tasks.preload_affected(&affected, &self.mimetype);
		true
	}
}
