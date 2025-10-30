use std::{ffi::{OsStr, OsString}, path::{Path, PathBuf}};

pub trait PathLike: AsRef<Self> {
	type Inner: ?Sized + PathInner;
	type Owned: PathBufLike + Into<Self::Owned>;
	type Components<'a>: AsRef<Self> + Clone + DoubleEndedIterator
	where
		Self: 'a;

	fn default() -> &'static Self;

	fn strip_prefix<T>(&self, base: T) -> Option<&Self>
	where
		T: AsRef<Self>;

	fn len(&self) -> usize { self.encoded_bytes().len() }

	fn components(&self) -> Self::Components<'_>;

	fn file_name(&self) -> Option<&Self::Inner>;

	fn file_stem(&self) -> Option<&Self::Inner>;

	fn extension(&self) -> Option<&Self::Inner>;

	fn parent(&self) -> Option<&Self>;

	fn encoded_bytes(&self) -> &[u8];

	unsafe fn from_encoded_bytes(bytes: &[u8]) -> &Self;

	fn join<T>(&self, base: T) -> Self::Owned
	where
		T: AsRef<Self>;
}

pub trait PathBufLike: AsRef<Self::Borrowed> + Default + 'static {
	type Inner: AsRef<Self::InnerRef>;
	type InnerRef: ?Sized + PathInner;
	type Borrowed: ?Sized + PathLike + AsRef<Self::Borrowed>;

	fn len(&self) -> usize { self.encoded_bytes().len() }

	fn encoded_bytes(&self) -> &[u8];

	fn into_encoded_bytes(self) -> Vec<u8>;

	unsafe fn from_encoded_bytes(bytes: Vec<u8>) -> Self;

	fn set_file_name<T>(&mut self, name: T)
	where
		T: AsRef<Self::InnerRef>;
}

pub trait PathInner {
	fn len(&self) -> usize { self.encoded_bytes().len() }

	fn encoded_bytes(&self) -> &[u8];
}

impl PathLike for Path {
	type Components<'a> = std::path::Components<'a>;
	type Inner = OsStr;
	type Owned = PathBuf;

	fn default() -> &'static Self { Path::new("") }

	fn strip_prefix<T>(&self, base: T) -> Option<&Self>
	where
		T: AsRef<Self>,
	{
		self.strip_prefix(base).ok()
	}

	fn components(&self) -> Self::Components<'_> { self.components() }

	fn file_name(&self) -> Option<&Self::Inner> { self.file_name() }

	fn file_stem(&self) -> Option<&Self::Inner> { self.file_stem() }

	fn extension(&self) -> Option<&Self::Inner> { self.extension() }

	fn parent(&self) -> Option<&Self> { self.parent() }

	fn encoded_bytes(&self) -> &[u8] { self.as_os_str().as_encoded_bytes() }

	unsafe fn from_encoded_bytes(bytes: &[u8]) -> &Self {
		Self::new(unsafe { Self::Inner::from_encoded_bytes_unchecked(bytes) })
	}

	fn join<T>(&self, base: T) -> Self::Owned
	where
		T: AsRef<Self>,
	{
		self.join(base)
	}
}

impl PathBufLike for PathBuf {
	type Borrowed = Path;
	type Inner = OsString;
	type InnerRef = OsStr;

	fn encoded_bytes(&self) -> &[u8] { self.as_os_str().as_encoded_bytes() }

	fn into_encoded_bytes(self) -> Vec<u8> { self.into_os_string().into_encoded_bytes() }

	unsafe fn from_encoded_bytes(bytes: Vec<u8>) -> Self {
		Self::from(unsafe { Self::Inner::from_encoded_bytes_unchecked(bytes) })
	}

	fn set_file_name<T>(&mut self, name: T)
	where
		T: AsRef<Self::InnerRef>,
	{
		self.set_file_name(name);
	}
}

impl PathInner for OsStr {
	fn encoded_bytes(&self) -> &[u8] { self.as_encoded_bytes() }
}
