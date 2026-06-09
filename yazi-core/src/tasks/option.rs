use std::borrow::Cow;

use yazi_macro::impl_data_any;
use yazi_scheduler::{TaskIn, file::{FileInCopy, FileInCut}, plugin::PluginInEntry};
use yazi_shared::{Id, SStr};

#[derive(Clone, Debug)]
pub enum TaskOpt {
	Cut(FileInCut),
	Copy(FileInCopy),

	Plugin(PluginInEntry),
}

impl_data_any!(TaskOpt);

impl TaskIn for TaskOpt {
	type Prog = ();

	fn id(&self) -> Id {
		match self {
			Self::Cut(r#in) => r#in.id(),
			Self::Copy(r#in) => r#in.id(),

			Self::Plugin(r#in) => r#in.id(),
		}
	}

	fn set_id(&mut self, id: Id) -> &mut Self {
		match self {
			Self::Cut(r#in) => _ = r#in.set_id(id),
			Self::Copy(r#in) => _ = r#in.set_id(id),

			Self::Plugin(r#in) => _ = r#in.set_id(id),
		}
		self
	}

	fn title(&self) -> Cow<'_, str> {
		match self {
			Self::Cut(r#in) => r#in.title(),
			Self::Copy(r#in) => r#in.title(),

			Self::Plugin(r#in) => r#in.title(),
		}
	}

	fn set_title(&mut self, title: impl Into<SStr>) -> &mut Self {
		match self {
			Self::Cut(r#in) => _ = r#in.set_title(title),
			Self::Copy(r#in) => _ = r#in.set_title(title),

			Self::Plugin(r#in) => _ = r#in.set_title(title),
		}
		self
	}
}
