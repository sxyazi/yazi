use crate::path::{AsPathView, PathBufLike, PathInner};

pub trait PathLike<'p>
where
	Self: Copy + AsPathView<'p, Self::View<'p>>,
{
	type Inner: PathInner<'p>;
	type Owned: PathBufLike + Into<Self::Owned>;
	type View<'a>;
	type Components<'a>: AsPathView<'a, Self::View<'a>> + Clone + DoubleEndedIterator;

	fn components(self) -> Self::Components<'p>;

	fn default() -> Self;

	fn encoded_bytes(self) -> &'p [u8];

	fn extension(self) -> Option<Self::Inner>;

	fn file_name(self) -> Option<Self::Inner>;

	fn file_stem(self) -> Option<Self::Inner>;

	unsafe fn from_encoded_bytes(bytes: &'p [u8]) -> Self;

	#[cfg(unix)]
	fn is_hidden(self) -> bool {
		self.file_name().map_or(false, |n| n.encoded_bytes().get(0) == Some(&b'.'))
	}

	fn join<'a, T>(self, base: T) -> Self::Owned
	where
		T: AsPathView<'a, Self::View<'a>>;

	fn len(self) -> usize { self.encoded_bytes().len() }

	fn parent(self) -> Option<Self>;

	fn strip_prefix<'a, T>(self, base: T) -> Option<Self>
	where
		T: AsPathView<'a, Self::View<'a>>;
}

impl<'p> PathLike<'p> for &'p std::path::Path {
	type Components<'a> = std::path::Components<'a>;
	type Inner = &'p std::ffi::OsStr;
	type Owned = std::path::PathBuf;
	type View<'a> = &'a std::path::Path;

	fn components(self) -> Self::Components<'p> { self.components() }

	fn default() -> Self { std::path::Path::new("") }

	fn encoded_bytes(self) -> &'p [u8] { self.as_os_str().as_encoded_bytes() }

	fn extension(self) -> Option<Self::Inner> { self.extension() }

	fn file_name(self) -> Option<Self::Inner> { self.file_name() }

	fn file_stem(self) -> Option<Self::Inner> { self.file_stem() }

	unsafe fn from_encoded_bytes(bytes: &'p [u8]) -> Self {
		std::path::Path::new(unsafe { std::ffi::OsStr::from_encoded_bytes_unchecked(bytes) })
	}

	fn join<'a, T>(self, base: T) -> Self::Owned
	where
		T: AsPathView<'a, Self::View<'a>>,
	{
		self.join(base.as_path_view())
	}

	fn parent(self) -> Option<Self> { self.parent() }

	fn strip_prefix<'a, T>(self, base: T) -> Option<Self>
	where
		T: AsPathView<'a, Self::View<'a>>,
	{
		self.strip_prefix(base.as_path_view()).ok()
	}
}
