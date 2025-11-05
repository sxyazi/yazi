use std::{fmt::Debug, hash::Hash};

use crate::path::{AsInnerView, AsPathView, PathInner, PathLike};

pub trait PathBufLike
where
	Self: 'static,
{
	type Inner: for<'a> AsInnerView<'a, Self::InnerRef<'a>>;
	type InnerRef<'a>: PathInner<'a>;
	type Borrowed<'a>: PathLike<'a> + AsPathView<'a, Self::Borrowed<'a>> + Debug + Hash;

	fn encoded_bytes(&self) -> &[u8];

	unsafe fn from_encoded_bytes(bytes: Vec<u8>) -> Self;

	fn into_encoded_bytes(self) -> Vec<u8>;

	fn len(&self) -> usize { self.encoded_bytes().len() }

	fn set_file_name<T>(&mut self, name: T)
	where
		T: for<'a> AsInnerView<'a, Self::InnerRef<'a>>;

	fn take(&mut self) -> Self;
}

impl PathBufLike for std::path::PathBuf {
	type Borrowed<'a> = &'a std::path::Path;
	type Inner = std::ffi::OsString;
	type InnerRef<'a> = &'a std::ffi::OsStr;

	fn encoded_bytes(&self) -> &[u8] { self.as_os_str().as_encoded_bytes() }

	unsafe fn from_encoded_bytes(bytes: Vec<u8>) -> Self {
		Self::from(unsafe { Self::Inner::from_encoded_bytes_unchecked(bytes) })
	}

	fn into_encoded_bytes(self) -> Vec<u8> { self.into_os_string().into_encoded_bytes() }

	fn set_file_name<T>(&mut self, name: T)
	where
		T: for<'a> AsInnerView<'a, Self::InnerRef<'a>>,
	{
		self.set_file_name(name.as_inner_view());
	}

	fn take(&mut self) -> Self { std::mem::take(self) }
}
