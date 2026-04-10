use std::borrow::Cow;

use yazi_shared::{Id, SStr};

pub trait TaskIn {
	type Prog;

	fn id(&self) -> Id;

	fn set_id(&mut self, id: Id) -> &mut Self;

	fn title(&self) -> Cow<'_, str>;

	fn set_title(&mut self, _title: impl Into<SStr>) -> &mut Self { self }
}
