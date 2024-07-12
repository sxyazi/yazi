use mlua::AnyUserData;

use crate::elements::Renderable;

pub fn cast_to_renderable(ud: &AnyUserData) -> Option<Box<dyn Renderable + Send>> {
	if let Ok(c) = ud.take::<crate::elements::Paragraph>() {
		Some(Box::new(c))
	} else if let Ok(c) = ud.take::<crate::elements::List>() {
		Some(Box::new(c))
	} else if let Ok(c) = ud.take::<crate::elements::Bar>() {
		Some(Box::new(c))
	} else if let Ok(c) = ud.take::<crate::elements::Clear>() {
		Some(Box::new(c))
	} else if let Ok(c) = ud.take::<crate::elements::Border>() {
		Some(Box::new(c))
	} else if let Ok(c) = ud.take::<crate::elements::Gauge>() {
		Some(Box::new(c))
	} else {
		None
	}
}
