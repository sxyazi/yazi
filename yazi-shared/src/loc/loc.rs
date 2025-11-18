use std::{hash::{Hash, Hasher}, marker::PhantomData, ops::Deref, path::Path};

use anyhow::{Result, bail};

use crate::{loc::LocBuf, path::{AsPathDyn, AsPathView, PathBufLike, PathDyn, PathLike, PathUnsafeExt}, scheme::SchemeKind, strand::{AsStrandView, StrandLike}};

#[derive(Clone, Copy, Debug)]
pub struct Loc<'p, P = &'p Path> {
	pub(super) inner:    P,
	pub(super) uri:      usize,
	pub(super) urn:      usize,
	pub(super) _phantom: PhantomData<&'p ()>,
}

impl<'p, P> Default for Loc<'p, P>
where
	P: PathLike<'p>,
{
	fn default() -> Self { Self { inner: P::default(), uri: 0, urn: 0, _phantom: PhantomData } }
}

impl<'p, P> Deref for Loc<'p, P>
where
	P: PathLike<'p>,
{
	type Target = P;

	fn deref(&self) -> &Self::Target { &self.inner }
}

impl<'p, P> AsPathDyn for Loc<'p, P>
where
	P: PathLike<'p> + AsPathDyn,
{
	fn as_path_dyn(&self) -> PathDyn<'_> { self.inner.as_path_dyn() }
}

// FIXME: remove
impl AsRef<std::path::Path> for Loc<'_, &std::path::Path> {
	fn as_ref(&self) -> &std::path::Path { self.inner }
}

// --- Hash
impl<'p, P> Hash for Loc<'p, P>
where
	P: PathLike<'p> + Hash,
{
	fn hash<H: Hasher>(&self, state: &mut H) { self.inner.hash(state) }
}

impl<'p, P> From<Loc<'p, P>> for LocBuf<<P as PathLike<'p>>::Owned>
where
	P: PathLike<'p>,
	<P as PathLike<'p>>::Owned: PathBufLike,
{
	fn from(value: Loc<'p, P>) -> Self {
		Self { inner: value.inner.owned(), uri: value.uri, urn: value.urn }
	}
}

// --- Eq
impl<'p, P> PartialEq for Loc<'p, P>
where
	P: PathLike<'p> + PartialEq,
{
	fn eq(&self, other: &Self) -> bool { self.inner == other.inner }
}

impl<'p, P> Eq for Loc<'p, P> where P: PathLike<'p> + Eq {}

impl<'p, P> Loc<'p, P>
where
	P: PathLike<'p> + PathUnsafeExt<'p>,
{
	pub fn new<'a, T, S>(path: T, base: S, trail: S) -> Self
	where
		T: AsPathView<'p, P>,
		S: AsStrandView<'a, P::Strand<'a>>,
	{
		let mut loc = Self::bare(path);
		loc.uri = loc.inner.try_strip_prefix(base).expect("Loc must start with the given base").len();
		loc.urn = loc.inner.try_strip_prefix(trail).expect("Loc must start with the given trail").len();
		loc
	}

	pub fn with<T>(path: T, uri: usize, urn: usize) -> Result<Self>
	where
		T: AsPathView<'p, P>,
	{
		if urn > uri {
			bail!("URN cannot be longer than URI");
		}

		let mut loc = Self::bare(path);
		if uri == 0 {
			(loc.uri, loc.urn) = (0, 0);
			return Ok(loc);
		} else if urn == 0 {
			loc.urn = 0;
		}

		let mut it = loc.inner.components();
		for i in 1..=uri {
			if it.next_back().is_none() {
				bail!("URI exceeds the entire URL");
			}
			if i == urn {
				loc.urn = loc.try_strip_prefix(it.clone())?.len();
			}
			if i == uri {
				loc.uri = loc.try_strip_prefix(it)?.len();
				break;
			}
		}
		Ok(loc)
	}

	pub fn bare<T>(path: T) -> Self
	where
		T: AsPathView<'p, P>,
	{
		let path = path.as_path_view();
		let Some(name) = path.name() else {
			let uri = path.len();
			return Self { inner: path, uri, urn: 0, _phantom: PhantomData };
		};

		let name_len = name.len();
		let prefix_len =
			unsafe { name.encoded_bytes().as_ptr().offset_from_unsigned(path.encoded_bytes().as_ptr()) };

		let bytes = path.encoded_bytes();
		Self {
			inner:    unsafe { P::from_encoded_bytes(&bytes[..prefix_len + name_len]) },
			uri:      name_len,
			urn:      name_len,
			_phantom: PhantomData,
		}
	}

	pub fn zeroed<T>(path: T) -> Self
	where
		T: AsPathView<'p, P>,
	{
		let mut loc = Self::bare(path);
		(loc.uri, loc.urn) = (0, 0);
		loc
	}

	pub fn floated<'a, T, S>(path: T, base: S) -> Self
	where
		T: AsPathView<'p, P>,
		S: AsStrandView<'a, P::Strand<'a>>,
	{
		let mut loc = Self::bare(path);
		loc.uri = loc.inner.try_strip_prefix(base).expect("Loc must start with the given base").len();
		loc
	}

	pub fn saturated<'a, T>(path: T, kind: SchemeKind) -> Self
	where
		T: AsPathView<'p, P>,
	{
		match kind {
			SchemeKind::Regular => Self::bare(path),
			SchemeKind::Search => Self::zeroed(path),
			SchemeKind::Archive => Self::zeroed(path),
			SchemeKind::Sftp => Self::bare(path),
		}
	}

	#[inline]
	pub fn as_loc(self) -> Self { self }

	#[inline]
	pub fn as_path(self) -> P { self.inner }

	#[inline]
	pub fn is_empty(self) -> bool { self.inner.is_empty() }

	#[inline]
	pub fn uri(self) -> P {
		unsafe {
			P::from_encoded_bytes(self.inner.encoded_bytes().get_unchecked(self.inner.len() - self.uri..))
		}
	}

	#[inline]
	pub fn urn(self) -> P {
		unsafe {
			P::from_encoded_bytes(self.inner.encoded_bytes().get_unchecked(self.inner.len() - self.urn..))
		}
	}

	#[inline]
	pub fn base(self) -> P {
		unsafe {
			P::from_encoded_bytes(self.inner.encoded_bytes().get_unchecked(..self.inner.len() - self.uri))
		}
	}

	#[inline]
	pub fn has_base(self) -> bool { self.inner.len() != self.uri }

	#[inline]
	pub fn trail(self) -> P {
		unsafe {
			P::from_encoded_bytes(self.inner.encoded_bytes().get_unchecked(..self.inner.len() - self.urn))
		}
	}

	#[inline]
	pub fn has_trail(self) -> bool { self.inner.len() != self.urn }

	#[inline]
	pub fn triple(self) -> (P, P, P) {
		let len = self.inner.len();

		let base = ..len - self.uri;
		let rest = len - self.uri..len - self.urn;
		let urn = len - self.urn..;

		unsafe {
			(
				P::from_encoded_bytes(self.inner.encoded_bytes().get_unchecked(base)),
				P::from_encoded_bytes(self.inner.encoded_bytes().get_unchecked(rest)),
				P::from_encoded_bytes(self.inner.encoded_bytes().get_unchecked(urn)),
			)
		}
	}
}
