use std::ops::Deref;

use crossterm::event::MouseButton;
use mlua::{UserData, UserDataFields};

#[derive(Clone, Copy)]
pub struct MouseEvent(crossterm::event::MouseEvent);

impl Deref for MouseEvent {
	type Target = crossterm::event::MouseEvent;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl From<crossterm::event::MouseEvent> for MouseEvent {
	fn from(event: crossterm::event::MouseEvent) -> Self { Self(event) }
}

impl UserData for MouseEvent {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("x", |_, me| Ok(me.column));
		fields.add_field_method_get("y", |_, me| Ok(me.row));
		fields.add_field_method_get("is_left", |_, me| {
			use crossterm::event::MouseEventKind as K;
			Ok(matches!(me.kind, K::Down(b) | K::Up(b) | K::Drag(b) if b == MouseButton::Left))
		});
		fields.add_field_method_get("is_right", |_, me| {
			use crossterm::event::MouseEventKind as K;
			Ok(matches!(me.kind, K::Down(b) | K::Up(b) | K::Drag(b) if b == MouseButton::Right))
		});
		fields.add_field_method_get("is_middle", |_, me| {
			use crossterm::event::MouseEventKind as K;
			Ok(matches!(me.kind, K::Down(b) | K::Up(b) | K::Drag(b) if b == MouseButton::Middle))
		});
	}
}
