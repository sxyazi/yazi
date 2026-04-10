use std::{hash::BuildHasher, path::{Path, PathBuf}};

use hashbrown::{HashMap, hash_map::EntryRef};

use crate::{loc::Loc, url::{Url, UrlBuf, UrlBufCov, UrlCov, UrlCow}};

// --- AsUrl
pub trait AsUrl {
	fn as_url(&self) -> Url<'_>;
}

impl AsUrl for Path {
	#[inline]
	fn as_url(&self) -> Url<'_> { Url::Regular(Loc::bare(self)) }
}

impl AsUrl for &Path {
	#[inline]
	fn as_url(&self) -> Url<'_> { (*self).as_url() }
}

impl AsUrl for PathBuf {
	#[inline]
	fn as_url(&self) -> Url<'_> { self.as_path().as_url() }
}

impl AsUrl for &PathBuf {
	#[inline]
	fn as_url(&self) -> Url<'_> { (*self).as_path().as_url() }
}

impl AsUrl for Url<'_> {
	#[inline]
	fn as_url(&self) -> Url<'_> { *self }
}

impl AsUrl for UrlBuf {
	#[inline]
	fn as_url(&self) -> Url<'_> {
		match self {
			Self::Regular(loc) => Url::Regular(loc.as_loc()),
			Self::Search { loc, domain } => Url::Search { loc: loc.as_loc(), domain },
			Self::Archive { loc, domain } => Url::Archive { loc: loc.as_loc(), domain },
			Self::Sftp { loc, domain } => Url::Sftp { loc: loc.as_loc(), domain },
		}
	}
}

impl AsUrl for &UrlBuf {
	#[inline]
	fn as_url(&self) -> Url<'_> { (**self).as_url() }
}

impl AsUrl for &mut UrlBuf {
	#[inline]
	fn as_url(&self) -> Url<'_> { (**self).as_url() }
}

impl AsUrl for UrlCow<'_> {
	fn as_url(&self) -> Url<'_> {
		match self {
			Self::Regular(loc) => Url::Regular(loc.as_loc()),
			Self::Search { loc, domain } => Url::Search { loc: loc.as_loc(), domain },
			Self::Archive { loc, domain } => Url::Archive { loc: loc.as_loc(), domain },
			Self::Sftp { loc, domain } => Url::Sftp { loc: loc.as_loc(), domain },
		}
	}
}

impl AsUrl for &UrlCow<'_> {
	fn as_url(&self) -> Url<'_> { (**self).as_url() }
}

impl AsUrl for super::Components<'_> {
	fn as_url(&self) -> Url<'_> { self.url() }
}

impl<'a, T> From<&'a T> for Url<'a>
where
	T: AsUrl + ?Sized,
{
	fn from(value: &'a T) -> Self { value.as_url() }
}

impl<'a, T> From<&'a mut T> for Url<'a>
where
	T: AsUrl + ?Sized,
{
	fn from(value: &'a mut T) -> Self { value.as_url() }
}

// --- UrlMapExt
pub trait UrlMapExt<V> {
	fn get_or_insert_default<U>(&mut self, url: U) -> &mut V
	where
		U: AsUrl,
		V: Default;

	fn get_or_insert_with<U, F>(&mut self, url: U, default: F) -> &mut V
	where
		U: AsUrl,
		F: FnOnce(Url<'_>) -> V;
}

impl<V, S> UrlMapExt<V> for HashMap<UrlBuf, V, S>
where
	S: BuildHasher,
{
	fn get_or_insert_default<U>(&mut self, url: U) -> &mut V
	where
		U: AsUrl,
		V: Default,
	{
		self.get_or_insert_with(url, |_| Default::default())
	}

	fn get_or_insert_with<U, F>(&mut self, url: U, default: F) -> &mut V
	where
		U: AsUrl,
		F: FnOnce(Url<'_>) -> V,
	{
		let url = url.as_url();
		match self.entry_ref(&url) {
			EntryRef::Occupied(oe) => oe.into_mut(),
			EntryRef::Vacant(ve) => ve.insert_with_key(url.into(), default(url)),
		}
	}
}

// --- UrlCovMapExt
pub trait UrlCovMapExt<V> {
	fn get_or_insert_default(&mut self, url: UrlCov<'_>) -> &mut V
	where
		V: Default;
}

impl<V, S> UrlCovMapExt<V> for HashMap<UrlBufCov, V, S>
where
	S: BuildHasher,
{
	fn get_or_insert_default(&mut self, url: UrlCov<'_>) -> &mut V
	where
		V: Default,
	{
		match self.entry_ref(&url) {
			EntryRef::Occupied(oe) => oe.into_mut(),
			EntryRef::Vacant(ve) => ve.insert_with_key(url.into(), Default::default()),
		}
	}
}
