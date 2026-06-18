use mlua::AnyUserData;
use yazi_shared::url::UrlBuf;

use super::Lives;
use crate::lives::PtrCell;

#[derive(Clone, Copy)]
pub(super) struct Selected;

impl Selected {
	pub(super) fn make(inner: &yazi_core::tab::Selected) -> mlua::Result<AnyUserData> {
		let inner = PtrCell::from(inner);

		Lives::scoped_userdata(yazi_binding::Iter::new(
			inner.as_static().values().map(UrlBuf::from),
			Some(inner.len()),
		))
	}
}
