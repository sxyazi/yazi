use std::ops::Deref;

use mlua::{UserData, UserDataFields};
use yazi_term::event::{MouseButton, MouseEventKind};

#[derive(Clone, Copy)]
pub struct MouseEvent(yazi_term::event::MouseEvent);

impl Deref for MouseEvent {
	type Target = yazi_term::event::MouseEvent;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl From<yazi_term::event::MouseEvent> for MouseEvent {
	fn from(event: yazi_term::event::MouseEvent) -> Self { Self(event) }
}

impl UserData for MouseEvent {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("x", |_, me| Ok(me.column));
		fields.add_field_method_get("y", |_, me| Ok(me.row));
		fields.add_field_method_get("is_left", |_, me| {
			use MouseEventKind as K;
			Ok(matches!(me.kind, K::Down(b) | K::Up(b) | K::Drag(b) if b == MouseButton::Left))
		});
		fields.add_field_method_get("is_right", |_, me| {
			use MouseEventKind as K;
			Ok(matches!(me.kind, K::Down(b) | K::Up(b) | K::Drag(b) if b == MouseButton::Right))
		});
		fields.add_field_method_get("is_middle", |_, me| {
			use MouseEventKind as K;
			Ok(matches!(me.kind, K::Down(b) | K::Up(b) | K::Drag(b) if b == MouseButton::Middle))
		});
	}
}
