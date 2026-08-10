use std::borrow::Cow;

use yazi_macro::impl_data_any;
use yazi_scheduler::{TaskIn, custom::CustomIn, file::{FileInCopy, FileInMove}, plugin::PluginInEntry};
use yazi_shared::id::Id;
use yazi_shim::SStr;

#[derive(Clone, Debug)]
pub enum TaskOpt {
	Copy(FileInCopy),
	Move(FileInMove),

	Plugin(PluginInEntry),

	Custom(CustomIn),
}

impl_data_any!(TaskOpt);

impl TaskIn for TaskOpt {
	type Prog = ();

	fn id(&self) -> Id {
		match self {
			Self::Copy(r#in) => r#in.id(),
			Self::Move(r#in) => r#in.id(),

			Self::Plugin(r#in) => r#in.id(),

			Self::Custom(r#in) => r#in.id(),
		}
	}

	fn set_id(&mut self, id: Id) -> &mut Self {
		match self {
			Self::Copy(r#in) => _ = r#in.set_id(id),
			Self::Move(r#in) => _ = r#in.set_id(id),

			Self::Plugin(r#in) => _ = r#in.set_id(id),

			Self::Custom(r#in) => _ = r#in.set_id(id),
		}
		self
	}

	fn title(&self) -> Cow<'_, str> {
		match self {
			Self::Copy(r#in) => r#in.title(),
			Self::Move(r#in) => r#in.title(),

			Self::Plugin(r#in) => r#in.title(),

			Self::Custom(r#in) => r#in.title(),
		}
	}

	fn set_title(&mut self, title: impl Into<SStr>) -> &mut Self {
		match self {
			Self::Copy(r#in) => _ = r#in.set_title(title),
			Self::Move(r#in) => _ = r#in.set_title(title),

			Self::Plugin(r#in) => _ = r#in.set_title(title),

			Self::Custom(r#in) => _ = r#in.set_title(title),
		}
		self
	}
}
