use super::Position;

pub struct InputOpt {
	pub title:      String,
	pub value:      String,
	pub position:   Position,
	pub realtime:   bool,
	pub completion: bool,
	pub highlight:  bool,
}

pub struct SelectOpt {
	pub title:    String,
	pub items:    Vec<String>,
	pub position: Position,
}

impl InputOpt {
	#[inline]
	pub fn cd() -> Self { todo!() }

	#[inline]
	pub fn create() -> Self { todo!() }

	#[inline]
	pub fn rename() -> Self { todo!() }

	#[inline]
	pub fn trash(n: usize) -> Self { todo!() }

	#[inline]
	pub fn delete(n: usize) -> Self { todo!() }

	#[inline]
	pub fn find(prev: bool) -> Self { todo!() }

	#[inline]
	pub fn search() -> Self { todo!() }

	#[inline]

	pub fn shell(block: bool) -> Self { todo!() }

	#[inline]
	pub fn overwrite() -> Self { todo!() }

	#[inline]
	pub fn with_value(mut self, value: impl Into<String>) -> Self {
		self.value = value.into();
		self
	}
}

impl SelectOpt {
	#[inline]
	pub fn open() -> Self { todo!() }

	#[inline]
	pub fn with_items(mut self, items: Vec<String>) -> Self {
		self.items = items;
		self
	}
}
