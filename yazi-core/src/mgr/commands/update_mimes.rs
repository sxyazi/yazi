use std::{borrow::Cow, collections::HashMap};

use tracing::error;
use yazi_macro::render;
use yazi_shared::{event::CmdCow, url::Url};

use crate::{mgr::{LINKED, Mgr}, tasks::Tasks};

pub struct Opt {
	updates: HashMap<Cow<'static, str>, String>,
}

impl TryFrom<CmdCow> for Opt {
	type Error = ();

	fn try_from(mut c: CmdCow) -> Result<Self, Self::Error> {
		Ok(Self { updates: c.try_take("updates").ok_or(())?.into_dict_string() })
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
			.map(|(url, mime)| (Url::from(url.into_owned()), mime))
			.filter(|(url, mime)| self.mimetype.by_url(url) != Some(mime))
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
