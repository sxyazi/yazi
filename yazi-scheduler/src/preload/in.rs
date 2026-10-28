use std::borrow::Cow;

use yazi_config::plugin::PreloaderArc;
use yazi_fs::file::File;
use yazi_shared::{id::Id, pool::{InternStr, Symbol}, scope::{Scope, ScopeLike}};

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

	fn scope(&self) -> &Scope {
		match self {
			Self::Preload(r#in) => r#in.scope(),
			Self::Custom(r#in) => r#in.scope(),
		}
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
pub struct PreloadInPreload {
	pub id:        Id,
	pub preloader: PreloaderArc,
	pub file:      File,
	pub hash:      u64,
	pub mime:      Symbol<str>,
	pub scope:     Scope,
}

impl PreloadInPreload {
	pub fn new<F, M>(preloader: PreloaderArc, file: F, hash: u64, mime: M) -> Self
	where
		F: Into<File>,
		M: AsRef<str>,
	{
		Self {
			id: Id::ZERO,
			preloader,
			file: file.into(),
			hash,
			mime: mime.intern(),
			scope: Scope::default(),
		}
	}

	pub fn scoped(mut self, context: &(impl ScopeLike + ?Sized)) -> Self {
		self.scope = context.scope().detached();
		self
	}
}

impl TaskIn for PreloadInPreload {
	type Prog = PreloadProg;

	fn id(&self) -> Id { self.id }

	fn set_id(&mut self, id: Id) -> &mut Self {
		self.id = id;
		self
	}

	fn scope(&self) -> &Scope { &self.scope }

	fn title(&self) -> Cow<'_, str> { format!("Run preloader '{}'", self.preloader.name).into() }
}
