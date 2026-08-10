use std::borrow::Cow;

use yazi_config::plugin::PreloaderArc;
use yazi_shared::{id::Id, pool::Symbol};

use crate::{TaskIn, custom::CustomIn, preload::PreloadProg};

#[derive(Clone, Debug)]
pub(crate) enum PreloadIn {
	Preload(PreloadInPreload),
	Custom(CustomIn),
}

impl TaskIn for PreloadIn {
	type Prog = ();

	fn id(&self) -> Id {
		match self {
			Self::Preload(r#in) => r#in.id(),
			Self::Custom(r#in) => r#in.id(),
		}
	}

	fn set_id(&mut self, id: Id) -> &mut Self {
		match self {
			Self::Preload(r#in) => _ = r#in.set_id(id),
			Self::Custom(r#in) => _ = r#in.set_id(id),
		}
		self
	}

	fn title(&self) -> Cow<'_, str> {
		match self {
			Self::Preload(r#in) => r#in.title(),
			Self::Custom(r#in) => r#in.title(),
		}
	}
}

impl_from_in!(Preload(PreloadInPreload), Custom(CustomIn));

#[derive(Clone, Debug)]
pub(crate) struct PreloadInPreload {
	pub(crate) id:        Id,
	pub(crate) preloader: PreloaderArc,
	pub(crate) file:      yazi_fs::file::File,
	pub(crate) mime:      Symbol<str>,
}

impl TaskIn for PreloadInPreload {
	type Prog = PreloadProg;

	fn id(&self) -> Id { self.id }

	fn set_id(&mut self, id: Id) -> &mut Self {
		self.id = id;
		self
	}

	fn title(&self) -> Cow<'_, str> { format!("Run preloader '{}'", self.preloader.name).into() }
}
