use std::{borrow::Cow, ffi::{OsStr, OsString}, iter::FusedIterator, ops::Not};

use crate::{loc::Loc, path, scheme::{Encode as EncodeScheme, SchemeCow, SchemeRef}, strand::{StrandBuf, StrandCow}, url::{Component, Encode as EncodeUrl, Url}};

#[derive(Clone)]
pub struct Components<'a> {
	inner:          path::Components<'a>,
	url:            Url<'a>,
	back_yields:    usize,
	scheme_yielded: bool,
}

impl<'a> From<Url<'a>> for Components<'a> {
	fn from(value: Url<'a>) -> Self {
		Self {
			inner:          value.loc().components(),
			url:            value,
			back_yields:    0,
			scheme_yielded: false,
		}
	}
}

impl<'a> Components<'a> {
	pub fn covariant(&self, other: &Self) -> bool {
		match (self.scheme_yielded, other.scheme_yielded) {
			(true, true) => {}
			(false, false) if self.scheme().covariant(other.scheme()) => {}
			_ => return false,
		}
		self.inner == other.inner
	}

	pub fn os_str(&self) -> Cow<'a, OsStr> {
		let Ok(os) = self.inner.strand().as_os() else {
			return OsString::from(EncodeUrl(self.url()).to_string()).into();
		};

		if self.url.is_regular() || self.scheme_yielded {
			return os.into();
		}

		let mut s = OsString::from(EncodeScheme(self.url()).to_string());
		s.reserve_exact(os.len());
		s.push(os);
		s.into()
	}

	pub fn scheme(&self) -> SchemeRef<'a> {
		let left = self.inner.clone().count();

		let (uri, urn) = SchemeCow::retrieve_ports(self.url);
		let (uri, urn) = (
			uri.saturating_sub(self.back_yields).min(left),
			urn.saturating_sub(self.back_yields).min(left),
		);

		match self.url {
			Url::Regular(_) => SchemeRef::Regular { uri, urn },
			Url::Search { domain, .. } => SchemeRef::Search { domain, uri, urn },
			Url::Archive { domain, .. } => SchemeRef::Archive { domain, uri, urn },
			Url::Sftp { domain, .. } => SchemeRef::Sftp { domain, uri, urn },
		}
	}

	pub fn strand(&self) -> StrandCow<'a> {
		let s = self.inner.strand();
		if self.url.is_regular() || self.scheme_yielded {
			return s.into();
		}

		let mut buf = StrandBuf::with_str(s.kind(), EncodeScheme(self.url()).to_string());
		buf.reserve_exact(s.len());
		buf.try_push(s).expect("strand with same kind");
		buf.into()
	}

	pub fn url(&self) -> Url<'a> {
		let path = self.inner.path();
		let (uri, urn) = self.scheme().ports();
		match self.url {
			Url::Regular(_) => Url::Regular(Loc::with(path.as_os().unwrap(), uri, urn).unwrap()),
			Url::Search { domain, .. } => {
				Url::Search { loc: Loc::with(path.as_os().unwrap(), uri, urn).unwrap(), domain }
			}
			Url::Archive { domain, .. } => {
				Url::Archive { loc: Loc::with(path.as_os().unwrap(), uri, urn).unwrap(), domain }
			}
			Url::Sftp { domain, .. } => {
				Url::Sftp { loc: Loc::with(path.as_unix().unwrap(), uri, urn).unwrap(), domain }
			}
		}
	}
}

impl<'a> Iterator for Components<'a> {
	type Item = Component<'a>;

	fn next(&mut self) -> Option<Self::Item> {
		if !self.scheme_yielded {
			self.scheme_yielded = true;
			Some(Component::Scheme(self.scheme()))
		} else {
			self.inner.next().map(Into::into)
		}
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		let (min, max) = self.inner.size_hint();
		let scheme = self.scheme_yielded.not() as usize;

		(min + scheme, max.map(|n| n + scheme))
	}
}

impl<'a> DoubleEndedIterator for Components<'a> {
	fn next_back(&mut self) -> Option<Self::Item> {
		if let Some(c) = self.inner.next_back() {
			self.back_yields += 1;
			Some(c.into())
		} else if !self.scheme_yielded {
			self.scheme_yielded = true;
			Some(Component::Scheme(self.scheme()))
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
		match (self.scheme_yielded, other.scheme_yielded) {
			(true, true) => true,
			(false, false) if self.scheme() == other.scheme() => true,
			_ => false,
		}
	}
}

// --- Tests
#[cfg(test)]
mod tests {
	use anyhow::Result;

	use crate::{scheme::SchemeRef, url::{Component, UrlBuf, UrlLike}};

	#[test]
	fn test_url() -> Result<()> {
		use Component::*;
		use SchemeRef as S;

		crate::init_tests();

		let search: UrlBuf = "search://keyword//root/projects/yazi".parse()?;
		assert_eq!(search.uri(), "");
		assert_eq!(search.scheme(), S::Search { domain: "keyword", uri: 0, urn: 0 });

		let src = search.try_join("src")?;
		assert_eq!(src.uri(), "src");
		assert_eq!(src.scheme(), S::Search { domain: "keyword", uri: 1, urn: 1 });

		let main = src.try_join("main.rs")?;
		assert_eq!(main.urn(), "src/main.rs");
		assert_eq!(main.scheme(), S::Search { domain: "keyword", uri: 2, urn: 2 });

		let mut it = main.components();
		assert_eq!(it.url().scheme(), S::Search { domain: "keyword", uri: 2, urn: 2 });
		assert_eq!(it.next_back(), Some(Normal("main.rs".into())));
		assert_eq!(it.url().scheme(), S::Search { domain: "keyword", uri: 1, urn: 1 });
		assert_eq!(it.next_back(), Some(Normal("src".into())));
		assert_eq!(it.url().scheme(), S::Search { domain: "keyword", uri: 0, urn: 0 });
		assert_eq!(it.next_back(), Some(Normal("yazi".into())));
		assert_eq!(it.url().scheme(), S::Search { domain: "keyword", uri: 0, urn: 0 });

		let mut it = main.components();
		assert_eq!(it.next(), Some(Scheme(S::Search { domain: "keyword", uri: 2, urn: 2 })));
		assert_eq!(it.next(), Some(RootDir));
		assert_eq!(it.next(), Some(Normal("root".into())));
		assert_eq!(it.next(), Some(Normal("projects".into())));
		assert_eq!(it.next(), Some(Normal("yazi".into())));
		assert_eq!(it.url().scheme(), S::Search { domain: "keyword", uri: 2, urn: 2 });
		assert_eq!(it.next(), Some(Normal("src".into())));
		assert_eq!(it.url().scheme(), S::Search { domain: "keyword", uri: 1, urn: 1 });
		assert_eq!(it.next_back(), Some(Normal("main.rs".into())));
		assert_eq!(it.url().scheme(), S::Search { domain: "keyword", uri: 0, urn: 0 });
		assert_eq!(it.next(), None);
		assert_eq!(it.url().scheme(), S::Search { domain: "keyword", uri: 0, urn: 0 });

		Ok(())
	}
}
