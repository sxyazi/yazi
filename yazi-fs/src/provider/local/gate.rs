use std::ops::{Deref, DerefMut};

#[derive(Default)]
pub struct Gate(tokio::fs::OpenOptions);

impl Deref for Gate {
	type Target = tokio::fs::OpenOptions;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl DerefMut for Gate {
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}
