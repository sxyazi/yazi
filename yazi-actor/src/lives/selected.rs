use mlua::AnyUserData;

use super::{Lives, PtrCell};

#[derive(Clone, Copy)]
pub(super) struct Selected;

impl Selected {
	pub(super) fn make(inner: &yazi_core::tab::Selected) -> mlua::Result<AnyUserData> {
		let inner = PtrCell::from(inner);

		Lives::scoped_userdata(yazi_binding::Iter::new(
			inner.as_static().files().cloned(),
			Some(inner.len()),
		))
	}
}
