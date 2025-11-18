use std::{fmt::Debug, hash::Hash};

use crate::{path::{AsPath, AsPathView, PathBufDyn, PathLike, SetNameError, StartsWithError}, strand::{AsStrandView, StrandLike}};

pub trait PathBufLike
where
	Self: 'static + AsPath,
{
	type Strand<'a>: StrandLike<'a>;
	type Borrowed<'a>: PathLike<'a> + AsPathView<'a, Self::Borrowed<'a>> + Debug + Hash;

	fn borrow(&self) -> Self::Borrowed<'_>;

	fn encoded_bytes(&self) -> &[u8] { self.borrow().encoded_bytes() }

	fn into_dyn(self) -> PathBufDyn;

	fn into_encoded_bytes(self) -> Vec<u8>;

	fn is_empty(&self) -> bool { self.borrow().is_empty() }

	fn len(&self) -> usize { self.borrow().len() }

	fn to_str(&self) -> Result<&str, std::str::Utf8Error> { self.borrow().to_str() }

	fn try_set_name<'a, T>(&mut self, name: T) -> Result<(), SetNameError>
	where
		T: AsStrandView<'a, Self::Strand<'a>>;

	fn try_starts_with<'a, T>(&self, base: T) -> Result<bool, StartsWithError>
	where
		T: AsStrandView<'a, Self::Strand<'a>>;

	fn take(&mut self) -> Self;
}

impl PathBufLike for std::path::PathBuf {
	type Borrowed<'a> = &'a std::path::Path;
	type Strand<'a> = &'a std::ffi::OsStr;

	fn borrow(&self) -> Self::Borrowed<'_> { self.as_path() }

	fn into_dyn(self) -> PathBufDyn { PathBufDyn::Os(self) }

	fn into_encoded_bytes(self) -> Vec<u8> { self.into_os_string().into_encoded_bytes() }

	fn try_set_name<'a, T>(&mut self, name: T) -> Result<(), SetNameError>
	where
		T: AsStrandView<'a, Self::Strand<'a>>,
	{
		Ok(self.set_file_name(name.as_strand_view()))
	}

	fn try_starts_with<'a, T>(&self, base: T) -> Result<bool, StartsWithError>
	where
		T: AsStrandView<'a, Self::Strand<'a>>,
	{
		Ok(self.starts_with(base.as_strand_view()))
	}

	fn take(&mut self) -> Self { std::mem::take(self) }
}
