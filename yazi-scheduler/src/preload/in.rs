use std::borrow::Cow;

use yazi_config::plugin::PreloaderArc;
use yazi_shared::{id::Id, pool::Symbol};

use crate::{TaskIn, preload::PreloadProg};

#[derive(Clone, Debug)]
pub(crate) struct PreloadIn {
	pub(crate) id:        Id,
	pub(crate) preloader: PreloaderArc,
	pub(crate) file:      yazi_fs::file::File,
	pub(crate) mime:      Symbol<str>,
}

impl TaskIn for PreloadIn {
	type Prog = PreloadProg;

	fn id(&self) -> Id { self.id }

	fn set_id(&mut self, id: Id) -> &mut Self {
		self.id = id;
		self
	}

	fn title(&self) -> Cow<'_, str> { format!("Run preloader '{}'", self.preloader.name).into() }
}
