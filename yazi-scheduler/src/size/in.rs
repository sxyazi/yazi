use std::{borrow::Cow, sync::Arc};

use yazi_shared::{Id, Throttle, url::{UrlBuf, UrlLike}};

use crate::{TaskIn, size::SizeProg};

#[derive(Debug)]
pub(crate) struct SizeIn {
	pub(crate) id:       Id,
	pub(crate) target:   UrlBuf,
	pub(crate) throttle: Arc<Throttle<(UrlBuf, u64)>>,
}

impl TaskIn for SizeIn {
	type Prog = SizeProg;

	fn id(&self) -> Id { self.id }

	fn set_id(&mut self, id: Id) -> &mut Self {
		self.id = id;
		self
	}

	fn title(&self) -> Cow<'_, str> { format!("Size '{}'", self.target.display()).into() }
}
