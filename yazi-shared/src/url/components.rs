use std::{borrow::Cow, ffi::{OsStr, OsString}, iter::FusedIterator, ops::Not};

use crate::{auth::AuthArc, loc::Loc, path, spec::{Encode as EncodeSpec, Spec}, strand::{StrandBuf, StrandCow}, url::{Component, Url}};

#[derive(Clone)]
pub struct Components<'a> {
	inner:        path::Components<'a>,
	url:          Url<'a>,
	auth_yields:  usize,
	back_yields:  usize,
	auth_yielded: bool,
}

impl<'a> From<Url<'a>> for Components<'a> {
	fn from(value: Url<'a>) -> Self {
		Self {
			inner:        value.loc().components(),
			url:          value,
			auth_yields:  0,
			back_yields:  0,
			auth_yielded: false,
		}
	}
}

impl<'a> Components<'a> {
	pub fn covariant(&self, other: &Self) -> bool {
		match (self.auth_yielded, other.auth_yielded) {
			(true, true) => {}
			(false, false) if self.auth().covariant(other.auth()) => {}
			_ => return false,
		}
		self.inner == other.inner
	}

	pub fn os_str(&self) -> Cow<'a, OsStr> {
		let Ok(os) = self.inner.strand().as_os() else {
			return OsString::from(self.url().encode().to_string()).into();
		};

		if self.url.is_regular() || self.auth_yielded {
			return os.into();
		}

		let mut s = OsString::from(EncodeSpec(self.url()).to_string());
		s.reserve_exact(os.len());
		s.push(os);
		s.into()
	}

	fn auth(&self) -> &'a AuthArc { self.url.auth() }

	fn ports(&self) -> (usize, usize) {
		let left = self.inner.clone().count();

		let (uri, urn) = Spec::retrieve_ports(self.url);
		let (uri, urn) = (
			uri.saturating_sub(self.back_yields).min(left),
			urn.saturating_sub(self.back_yields).min(left),
		);

		(uri, urn)
	}

	pub fn strand(&self) -> StrandCow<'a> {
		let s = self.inner.strand();
		if self.url.is_regular() || self.auth_yielded {
			return s.into();
		}

		let mut buf = StrandBuf::with_str(s.kind(), EncodeSpec(self.url()).to_string());
		buf.reserve_exact(s.len());
		buf.try_push(s).expect("strand with same kind");
		buf.into()
	}

	pub(crate) fn url(&self) -> Url<'a> {
		let path = self.inner.path();
		let (uri, urn) = self.ports();
		match self.url {
			Url::Os { auth, .. } if auth.kind.is_hub() => Url::Os {
				loc:  Loc::with(path.as_os().unwrap(), uri, urn).unwrap(),
				auth: auth.parent_at(self.auth_yields),
			},
			Url::Os { auth, .. } => {
				Url::Os { loc: Loc::with(path.as_os().unwrap(), uri, urn).unwrap(), auth }
			}
			Url::Unix { auth, .. } => {
				Url::Unix { loc: Loc::with(path.as_unix().unwrap(), uri, urn).unwrap(), auth }
			}
		}
	}
}

impl<'a> Iterator for Components<'a> {
	type Item = Component<'a>;

	fn next(&mut self) -> Option<Self::Item> {
		if !self.auth_yielded {
			self.auth_yielded = true;
			Some(Component::Auth(self.auth()))
		} else {
			self.inner.next().map(Into::into)
		}
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		let (min, max) = self.inner.size_hint();
		let auth = self.auth_yielded.not() as usize;

		(min + auth, max.map(|n| n + auth))
	}
}

impl<'a> DoubleEndedIterator for Components<'a> {
	fn next_back(&mut self) -> Option<Self::Item> {
		if let Some(c) = self.inner.next_back() {
			self.auth_yields += c.has_auth() as usize;
			self.back_yields += 1;
			Some(c.into())
		} else if !self.auth_yielded {
			self.auth_yielded = true;
			Some(Component::Auth(self.auth()))
		} else {
			None
		}
	}
}

impl<'a> FusedIterator for Components<'a> {}

impl<'a> PartialEq for Components<'a> {
	fn eq(&self, other: &Self) -> bool {
		if self.inner != other.inner {
			return false;
		}
		match (self.auth_yielded, other.auth_yielded) {
			(true, true) => true,
			(false, false) if self.auth() == other.auth() => true,
			_ => false,
		}
	}
}

// --- Tests
#[cfg(test)]
mod tests {
	use anyhow::Result;

	use crate::url::{Component, UrlBuf, UrlLike};

	#[test]
	fn test_url() -> Result<()> {
		use Component::*;

		crate::init_tests();

		let view: UrlBuf = "test-view://fx/@Ds2kw0A//root/projects/yazi".parse()?;
		let s = |uri, urn| view.spec().with_ports(uri, urn);

		assert_eq!(view.uri(), "");
		assert_eq!(view.spec(), s(0, 0));

		let src = view.try_join("src")?;
		assert_eq!(src.uri(), "src");
		assert_eq!(src.spec(), s(1, 1));

		let main = src.try_join("main.rs")?;
		assert_eq!(main.urn(), "src/main.rs");
		assert_eq!(main.spec(), s(2, 2));

		let mut it = main.components();
		assert_eq!(it.url().spec(), s(2, 2));
		assert_eq!(it.next_back(), Some(Normal("main.rs".into())));
		assert_eq!(it.url().spec(), s(1, 1));
		assert_eq!(it.next_back(), Some(Normal("src".into())));
		assert_eq!(it.url().spec(), s(0, 0));
		assert_eq!(it.next_back(), Some(Normal("yazi".into())));
		assert_eq!(it.url().spec(), s(0, 0));

		let mut it = main.components();
		assert_eq!(it.next(), Some(Auth(view.auth())));
		assert_eq!(it.next(), Some(RootDir));
		assert_eq!(it.next(), Some(Normal("root".into())));
		assert_eq!(it.next(), Some(Normal("projects".into())));
		assert_eq!(it.next(), Some(Normal("yazi".into())));
		assert_eq!(it.url().spec(), s(2, 2));
		assert_eq!(it.next(), Some(Normal("src".into())));
		assert_eq!(it.url().spec(), s(1, 1));
		assert_eq!(it.next_back(), Some(Normal("main.rs".into())));
		assert_eq!(it.url().spec(), s(0, 0));
		assert_eq!(it.next(), None);
		assert_eq!(it.url().spec(), s(0, 0));

		Ok(())
	}
}
