use std::{ffi::{OsStr, OsString}, fmt::Debug, hash::Hash};

use crate::{path::{AsPath, AsPathView}, strand::AsStrandView};

// --- LocAble
pub trait LocAble<'p>
where
	Self: Copy + AsStrandView<'p, Self::Strand<'p>>,
{
	type Strand<'a>: StrandAble<'a> + StrandAbleImpl<'a>;
	type Owned: LocBufAble + LocBufAbleImpl + Into<Self::Owned>;
	type Components<'a>: Clone + DoubleEndedIterator + AsStrandView<'a, Self::Strand<'a>>;
}

impl<'p> LocAble<'p> for &'p std::path::Path {
	type Components<'a> = std::path::Components<'a>;
	type Owned = std::path::PathBuf;
	type Strand<'a> = &'a OsStr;
}

impl<'p> LocAble<'p> for &'p typed_path::UnixPath {
	type Components<'a> = typed_path::UnixComponents<'a>;
	type Owned = typed_path::UnixPathBuf;
	type Strand<'a> = &'a [u8];
}

// --- LocBufAble
pub trait LocBufAble
where
	Self: 'static + AsPath + Default,
{
	type Strand<'a>: StrandAble<'a>;
	type Borrowed<'a>: LocAble<'a>
		+ LocAbleImpl<'a>
		+ AsPathView<'a, Self::Borrowed<'a>>
		+ Debug
		+ Hash;
}

impl LocBufAble for std::path::PathBuf {
	type Borrowed<'a> = &'a std::path::Path;
	type Strand<'a> = &'a OsStr;
}

impl LocBufAble for typed_path::UnixPathBuf {
	type Borrowed<'a> = &'a typed_path::UnixPath;
	type Strand<'a> = &'a [u8];
}

// --- StrandAble
pub trait StrandAble<'a>: Copy {}

impl<'a> StrandAble<'a> for &'a OsStr {}

impl<'a> StrandAble<'a> for &'a [u8] {}

// --- LocAbleImpl
pub(super) trait LocAbleImpl<'p>: LocAble<'p> {
	fn as_encoded_bytes(self) -> &'p [u8];

	fn components(self) -> Self::Components<'p>;

	fn empty() -> Self;

	fn file_name(self) -> Option<Self::Strand<'p>>;

	unsafe fn from_encoded_bytes_unchecked(bytes: &'p [u8]) -> Self;

	fn join<'a, T>(self, path: T) -> Self::Owned
	where
		T: AsStrandView<'a, Self::Strand<'a>>;

	fn len(self) -> usize { self.as_encoded_bytes().len() }

	fn parent(self) -> Option<Self>;

	fn strip_prefix<'a, T>(self, base: T) -> Option<Self>
	where
		T: AsStrandView<'a, Self::Strand<'a>>;

	fn to_path_buf(self) -> Self::Owned;
}

impl<'p> LocAbleImpl<'p> for &'p std::path::Path {
	fn as_encoded_bytes(self) -> &'p [u8] { self.as_os_str().as_encoded_bytes() }

	fn components(self) -> Self::Components<'p> { self.components() }

	fn empty() -> Self { std::path::Path::new("") }

	fn file_name(self) -> Option<Self::Strand<'p>> { self.file_name() }

	unsafe fn from_encoded_bytes_unchecked(bytes: &'p [u8]) -> Self {
		std::path::Path::new(unsafe { OsStr::from_encoded_bytes_unchecked(bytes) })
	}

	fn join<'a, T>(self, path: T) -> Self::Owned
	where
		T: AsStrandView<'a, Self::Strand<'a>>,
	{
		self.join(path.as_strand_view())
	}

	fn parent(self) -> Option<Self> { self.parent() }

	fn strip_prefix<'a, T>(self, base: T) -> Option<Self>
	where
		T: AsStrandView<'a, Self::Strand<'a>>,
	{
		use std::path::is_separator;

		let p = self.strip_prefix(base.as_strand_view()).ok()?;
		let mut b = p.as_encoded_bytes();

		if b.last().is_none_or(|&c| !is_separator(c as char)) || p.parent().is_none() {
			return Some(p);
		}

		while let [head @ .., last] = b
			&& is_separator(*last as char)
		{
			b = head;
		}

		Some(unsafe { Self::from_encoded_bytes_unchecked(b) })
	}

	fn to_path_buf(self) -> Self::Owned { self.to_path_buf() }
}

impl<'p> LocAbleImpl<'p> for &'p typed_path::UnixPath {
	fn as_encoded_bytes(self) -> &'p [u8] { self.as_bytes() }

	fn components(self) -> Self::Components<'p> { self.components() }

	fn empty() -> Self { typed_path::UnixPath::new("") }

	fn file_name(self) -> Option<Self::Strand<'p>> { self.file_name() }

	unsafe fn from_encoded_bytes_unchecked(bytes: &'p [u8]) -> Self {
		typed_path::UnixPath::new(bytes)
	}

	fn join<'a, T>(self, path: T) -> Self::Owned
	where
		T: AsStrandView<'a, Self::Strand<'a>>,
	{
		self.join(path.as_strand_view())
	}

	fn parent(self) -> Option<Self> { self.parent() }

	fn strip_prefix<'a, T>(self, base: T) -> Option<Self>
	where
		T: AsStrandView<'a, Self::Strand<'a>>,
	{
		let p = self.strip_prefix(base.as_strand_view()).ok()?;
		let mut b = p.as_bytes();

		if b.last().is_none_or(|&c| c != b'/') || p.parent().is_none() {
			return Some(p);
		}

		while let [head @ .., b'/'] = b {
			b = head;
		}

		Some(typed_path::UnixPath::new(b))
	}

	fn to_path_buf(self) -> Self::Owned { self.to_path_buf() }
}

// --- LocBufAbleImpl
pub(super) trait LocBufAbleImpl: LocBufAble {
	fn as_encoded_bytes(&self) -> &[u8] { self.borrow().as_encoded_bytes() }

	fn borrow(&self) -> Self::Borrowed<'_>;

	unsafe fn from_encoded_bytes_unchecked(bytes: Vec<u8>) -> Self;

	fn into_encoded_bytes(self) -> Vec<u8>;

	fn len(&self) -> usize { self.borrow().len() }

	fn set_file_name<'a, T>(&mut self, name: T)
	where
		T: AsStrandView<'a, Self::Strand<'a>>;
}

impl LocBufAbleImpl for std::path::PathBuf {
	fn borrow(&self) -> Self::Borrowed<'_> { self.as_path() }

	unsafe fn from_encoded_bytes_unchecked(bytes: Vec<u8>) -> Self {
		Self::from(unsafe { OsString::from_encoded_bytes_unchecked(bytes) })
	}

	fn into_encoded_bytes(self) -> Vec<u8> { self.into_os_string().into_encoded_bytes() }

	fn set_file_name<'a, T>(&mut self, name: T)
	where
		T: AsStrandView<'a, Self::Strand<'a>>,
	{
		self.set_file_name(name.as_strand_view())
	}
}

impl LocBufAbleImpl for typed_path::UnixPathBuf {
	fn borrow(&self) -> Self::Borrowed<'_> { self.as_path() }

	unsafe fn from_encoded_bytes_unchecked(bytes: Vec<u8>) -> Self { bytes.into() }

	fn into_encoded_bytes(self) -> Vec<u8> { self.into_vec() }

	fn set_file_name<'a, T>(&mut self, name: T)
	where
		T: AsStrandView<'a, Self::Strand<'a>>,
	{
		self.set_file_name(name.as_strand_view())
	}
}

// --- StrandAbleImpl
pub(super) trait StrandAbleImpl<'a>: StrandAble<'a> {
	fn as_encoded_bytes(self) -> &'a [u8];

	fn len(self) -> usize { self.as_encoded_bytes().len() }
}

impl<'a> StrandAbleImpl<'a> for &'a OsStr {
	fn as_encoded_bytes(self) -> &'a [u8] { self.as_encoded_bytes() }
}

impl<'a> StrandAbleImpl<'a> for &'a [u8] {
	fn as_encoded_bytes(self) -> &'a [u8] { self }
}
