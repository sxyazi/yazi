use std::collections::HashMap;

use tracing::error;
use yazi_macro::render;
use yazi_shared::event::{CmdCow, Data, DataKey};

use crate::{mgr::{LINKED, Mgr}, tasks::Tasks};

pub struct Opt {
	updates: HashMap<DataKey, Data>,
}

impl TryFrom<CmdCow> for Opt {
	type Error = ();

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> {
		Ok(Self { updates: c.try_take("updates").and_then(Data::into_dict).ok_or(())? })
	}
}

impl Mgr {
	pub fn update_mimes(&mut self, opt: impl TryInto<Opt>, tasks: &Tasks) {
		let Ok(opt): Result<Opt, _> = opt.try_into() else {
			return error!("invalid arguments for update_mimes");
		};

		let linked = LINKED.read();
		let updates = opt
			.updates
			.into_iter()
			.flat_map(|(key, value)| key.into_url().zip(value.into_string()))
			.filter(|(url, mime)| self.mimetype.by_url(url) != Some(mime))
			.fold(HashMap::new(), |mut map, (u, m)| {
				for u in linked.from_file(&u) {
					map.insert(u, m.to_string());
				}
				map.insert(u, m.into_owned());
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

		let repeek = self.hovered().is_some_and(|f| updates.contains_key(&f.url));
		self.mimetype.extend(updates);

		if repeek {
			self.peek(false);
		}
		tasks.fetch_paged(&affected, &self.mimetype);
		tasks.preload_paged(&affected, &self.mimetype);

		render!();
	}
}
