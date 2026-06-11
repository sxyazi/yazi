use mlua::Value;
use tracing::error;

use crate::elements::Renderable;

pub struct Renderables;

impl Renderables {
	pub fn reduce<F>(value: Value, mut reducer: F)
	where
		F: FnMut(Renderable),
	{
		match value {
			Value::Table(tbl) => {
				for element in tbl.sequence_values::<Renderable>() {
					match element {
						Ok(r) => reducer(r),
						Err(e) => error!("Failed to convert to renderable elements: {e}"),
					}
				}
			}
			Value::UserData(ud) => match Renderable::try_from(&ud) {
				Ok(w) => reducer(w),
				Err(e) => error!("Failed to convert to renderable element: {e}"),
			},
			_ => error!("Expected a renderable element, or a table of them, got: {value:?}"),
		}
	}
}
