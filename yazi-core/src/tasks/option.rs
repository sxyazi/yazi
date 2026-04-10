use std::borrow::Cow;

use yazi_scheduler::{TaskIn, plugin::PluginInEntry};
use yazi_shared::{Id, SStr};

#[derive(Clone, Debug)]
pub enum TaskOpt {
	Plugin(PluginInEntry),
}

impl TaskIn for TaskOpt {
	type Prog = ();

	fn id(&self) -> Id {
		match self {
			Self::Plugin(r#in) => r#in.id(),
		}
	}

	fn set_id(&mut self, id: Id) -> &mut Self {
		match self {
			Self::Plugin(r#in) => _ = r#in.set_id(id),
		}
		self
	}

	fn title(&self) -> Cow<'_, str> {
		match self {
			Self::Plugin(r#in) => r#in.title(),
		}
	}

	fn set_title(&mut self, title: impl Into<SStr>) -> &mut Self {
		match self {
			Self::Plugin(r#in) => _ = r#in.set_title(title),
		}
		self
	}
}
