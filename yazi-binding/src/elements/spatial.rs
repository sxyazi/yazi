use crate::elements::Area;

pub trait Spatial {
	fn area(&self) -> Area;

	fn set_area(&mut self, area: Area);
}
