use std::borrow::Cow;

use yazi_config::plugin::Preloader;
use yazi_shared::Id;

use crate::{TaskIn, preload::PreloadProg};

#[derive(Clone, Debug)]
pub(crate) struct PreloadIn {
	pub(crate) id:     Id,
	pub(crate) plugin: &'static Preloader,
	pub(crate) target: yazi_fs::File,
}

impl TaskIn for PreloadIn {
	type Prog = PreloadProg;

	fn id(&self) -> Id { self.id }

	fn set_id(&mut self, id: Id) -> &mut Self {
		self.id = id;
		self
	}

	fn title(&self) -> Cow<'_, str> { format!("Run preloader '{}'", self.plugin.run.name).into() }
}
