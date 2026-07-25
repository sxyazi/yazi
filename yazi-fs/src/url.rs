use std::{borrow::Cow, path::{Path, PathBuf}};

use mlua::UserDataFields;
use yazi_shared::url::{AsUrl, Url, UrlBuf, UrlBufInventory, UrlCow, UrlLike};
use yazi_shim::mlua::UserDataFieldsExt;

use crate::{FsAuth, FsHash128};

pub trait FsUrl<'a> {
	fn cache_bucket(&self) -> Option<PathBuf>;

	fn cache_entry(&self) -> Option<PathBuf>;

	fn stamp_entry(&self) -> Option<PathBuf>;

	fn working_path(self) -> Cow<'a, Path>;
}

impl<'a> FsUrl<'a> for Url<'a> {
	fn cache_bucket(&self) -> Option<PathBuf> {
		self.auth().cache_root().map(|mut root| {
			root.push(self.hash_u128_str(&mut [0; 26]));
			root
		})
	}

	fn cache_entry(&self) -> Option<PathBuf> {
		let parent = self.parent()?;
		parent.auth().cache_root().map(|mut root| {
			root.push(parent.hash_u128_str(&mut [0; 26]));
			root.push(self.hash_u128_str(&mut [0; 26]));
			root
		})
	}

	fn stamp_entry(&self) -> Option<PathBuf> {
		self.auth().stamp_root().map(|mut root| {
			root.push(self.hash_u128_str(&mut [0; 26]));
			root
		})
	}

	fn working_path(self) -> Cow<'a, Path> {
		match self {
			Self::Regular(loc) | Self::Search { loc, .. } => loc.as_inner().into(),
			Self::Mount { .. } | Self::Hub { .. } | Self::Scope { .. } | Self::Sftp { .. } => {
				self.cache_bucket().expect("non-local URL should have a cache path").into()
			}
		}
	}
}

impl FsUrl<'_> for UrlBuf {
	fn cache_bucket(&self) -> Option<PathBuf> { self.as_url().cache_bucket() }

	fn cache_entry(&self) -> Option<PathBuf> { self.as_url().cache_entry() }

	fn stamp_entry(&self) -> Option<PathBuf> { self.as_url().stamp_entry() }

	fn working_path(self) -> Cow<'static, Path> {
		match self {
			Self::Regular(loc) | Self::Search { loc, .. } => loc.into_inner().into(),
			Self::Mount { .. } | Self::Hub { .. } | Self::Scope { .. } | Self::Sftp { .. } => {
				self.cache_bucket().expect("non-local URL should have a cache path").into()
			}
		}
	}
}

impl<'a> FsUrl<'a> for UrlCow<'a> {
	fn cache_bucket(&self) -> Option<PathBuf> { self.as_url().cache_bucket() }

	fn cache_entry(&self) -> Option<PathBuf> { self.as_url().cache_entry() }

	fn stamp_entry(&self) -> Option<PathBuf> { self.as_url().stamp_entry() }

	fn working_path(self) -> Cow<'a, Path> {
		match self {
			Self::Regular(loc) | Self::Search { loc, .. } => loc.into_inner(),
			Self::Mount { .. } | Self::Hub { .. } | Self::Scope { .. } | Self::Sftp { .. } => {
				self.cache_bucket().expect("non-local URL should have a cache path").into()
			}
		}
	}
}

// --- Inject
inventory::submit! {
	UrlBufInventory {
		register: |registry| {
			registry.add_cached_field("domain", |lua, me| {
				yazi_binding::deprecate!(lua, "{}: `Url.domain` is deprecated, use `Url.spec.domain` instead.");
				lua.create_string(&*me.spec().domain)
			});
			registry.add_field_method_get("is_regular", |lua, me| {
				yazi_binding::deprecate!(lua, "{}: `Url.is_regular` is deprecated, use `Url.spec.is_regular` instead.");
				Ok(me.is_regular())
			});
			registry.add_field_method_get("is_search", |lua, me| {
				yazi_binding::deprecate!(lua, "{}: `Url.is_search` is deprecated, use `Url.spec.is_search` instead.");
				Ok(me.is_search())
			});
			registry.add_cached_field("scheme", |lua, me| {
				yazi_binding::deprecate!(lua, "{}: `Url.scheme` is deprecated, use `Url.spec` instead.");
				Ok(me.spec())
			});
		}
	}
}
