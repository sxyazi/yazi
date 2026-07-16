use std::{borrow::Cow, ffi::OsStr, path::{Path, PathBuf}};

use mlua::UserDataFields;
use yazi_shared::{path::{DynPath, PathDyn}, url::{AsUrl, Url, UrlBuf, UrlBufInventory, UrlCow, UrlLike}};
use yazi_shim::mlua::UserDataFieldsExt;

use crate::{FsHash128, FsSpec, path::PercentEncoding};

pub trait FsUrl<'a> {
	fn cache(&self) -> Option<PathBuf>;

	fn cache_lock(&self) -> Option<PathBuf>;

	fn unified_path(self) -> Cow<'a, Path>;

	fn unified_path_str(self) -> Cow<'a, OsStr>
	where
		Self: Sized,
	{
		match self.unified_path() {
			Cow::Borrowed(p) => p.as_os_str().into(),
			Cow::Owned(p) => p.into_os_string().into(),
		}
	}
}

impl<'a> FsUrl<'a> for Url<'a> {
	fn cache(&self) -> Option<PathBuf> {
		fn with_loc(loc: PathDyn, mut root: PathBuf) -> PathBuf {
			let mut it = loc.components();
			if it.next() == Some(yazi_shared::path::Component::RootDir) {
				root.push(it.dyn_path().percent_encode());
			} else {
				root.push(".%2F");
				root.push(loc.percent_encode());
			}
			root
		}

		self.auth().cache().map(|root| with_loc(self.loc(), root))
	}

	fn cache_lock(&self) -> Option<PathBuf> {
		self.auth().cache().map(|mut root| {
			root.push("%lock");
			root.push(format!("{:x}", self.hash_u128()));
			root
		})
	}

	fn unified_path(self) -> Cow<'a, Path> {
		match self {
			Self::Regular(loc) | Self::Search { loc, .. } => loc.as_inner().into(),
			Self::Mount { .. } | Self::Hub { .. } | Self::Scope { .. } | Self::Sftp { .. } => {
				self.cache().expect("non-local URL should have a cache path").into()
			}
		}
	}
}

impl FsUrl<'_> for UrlBuf {
	fn cache(&self) -> Option<PathBuf> { self.as_url().cache() }

	fn cache_lock(&self) -> Option<PathBuf> { self.as_url().cache_lock() }

	fn unified_path(self) -> Cow<'static, Path> {
		match self {
			Self::Regular(loc) | Self::Search { loc, .. } => loc.into_inner().into(),
			Self::Mount { .. } | Self::Hub { .. } | Self::Scope { .. } | Self::Sftp { .. } => {
				self.cache().expect("non-local URL should have a cache path").into()
			}
		}
	}
}

impl<'a> FsUrl<'a> for UrlCow<'a> {
	fn cache(&self) -> Option<PathBuf> { self.as_url().cache() }

	fn cache_lock(&self) -> Option<PathBuf> { self.as_url().cache_lock() }

	fn unified_path(self) -> Cow<'a, Path> {
		match self {
			Self::Regular(loc) | Self::Search { loc, .. } => loc.into_inner(),
			Self::Mount { .. } | Self::Hub { .. } | Self::Scope { .. } | Self::Sftp { .. } => {
				self.cache().expect("non-local URL should have a cache path").into()
			}
		}
	}
}

// --- Inject
inventory::submit! {
	UrlBufInventory {
		register: |registry| {
			registry.add_cached_field("cache", |_, me| Ok(me.cache()));
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
