use std::collections::HashMap;

use tracing::error;
use yazi_shared::{event::Cmd, fs::Url, render};

use crate::{manager::{Manager, LINKED}, tasks::Tasks};

pub struct Opt {
	updates: HashMap<String, String>,
}

impl TryFrom<Cmd> for Opt {
	type Error = ();

	fn try_from(mut c: Cmd) -> Result<Self, Self::Error> {
		Ok(Self { updates: c.take("updates").ok_or(())?.into_dict_string() })
	}
}

impl Manager {
	pub fn update_mimetype(&mut self, opt: impl TryInto<Opt>, tasks: &Tasks) {
		let Ok(opt) = opt.try_into() else {
			return error!("invalid arguments for update_mimetype");
		};

		let linked = LINKED.read();
		let updates = opt
			.updates
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
			return;
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

		tasks.prework_affected(&affected, &self.mimetype);
		render!();
	}
}
