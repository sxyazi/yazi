use std::{ops::Deref, path::{Component, Path}};

#[derive(Debug, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct Uri(Path);

impl Uri {
	#[inline]
	pub fn new<T: AsRef<Path> + ?Sized>(p: &T) -> &Self {
		unsafe { &*(p.as_ref() as *const Path as *const Self) }
	}

	#[inline]
	pub fn count(&self) -> usize { self.0.components().count() }

	#[inline]
	pub fn nth(&self, n: usize) -> Option<Component<'_>> { self.0.components().nth(n) }

	#[inline]
	pub fn is_empty(&self) -> bool { self.0.as_os_str().is_empty() }
}

impl Deref for Uri {
	type Target = Path;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl AsRef<Path> for Uri {
	fn as_ref(&self) -> &Path { &self.0 }
}
