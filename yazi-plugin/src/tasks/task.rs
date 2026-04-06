use mlua::{UserData, UserDataFields};
use yazi_shared::Id;

#[derive(Clone, Debug)]
pub(crate) struct Task {
	pub(super) id: Id,
}

impl UserData for Task {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("id", |_, me| Ok(yazi_binding::Id(me.id)));
	}
}
