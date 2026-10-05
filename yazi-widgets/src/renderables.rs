use mlua::Value;
use yazi_macro::{error, log_if_err};

use crate::Renderable;

pub struct Renderables;

impl Renderables {
	pub fn reduce<F>(value: Value, mut reducer: F)
	where
		F: FnMut(Renderable),
	{
		match value {
			Value::Table(tbl) => {
				for element in tbl.sequence_values::<Renderable>() {
					log_if_err!("Converting renderable elements", element.map(&mut reducer));
				}
			}
			Value::UserData(ud) => {
				log_if_err!("Converting renderable element", Renderable::try_from(&ud).map(&mut reducer));
			}
			_ => error!("Expected a renderable element, or a table of them, got: {value:?}"),
		}
	}
}
