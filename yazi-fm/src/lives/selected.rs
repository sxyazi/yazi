use indexmap::IndexMap;
use mlua::AnyUserData;

use super::Lives;
use crate::lives::PtrCell;

#[derive(Clone, Copy)]
pub(super) struct Selected;

impl Selected {
	#[inline]
	pub(super) fn make(inner: &IndexMap<yazi_shared::url::Url, u64>) -> mlua::Result<AnyUserData> {
		let inner = PtrCell::from(inner);

		Lives::scoped_userdata(yazi_binding::Iter::new(
			inner.as_static().keys().cloned().map(yazi_binding::Url::new),
			Some(inner.len()),
		))
	}
}
