use std::borrow::Cow;

use yazi_shared::Id;

pub(crate) trait TaskIn {
	type Prog;

	fn id(&self) -> Id;

	fn with_id(&mut self, id: Id) -> &mut Self;

	fn title(&self) -> Cow<'_, str>;
}
