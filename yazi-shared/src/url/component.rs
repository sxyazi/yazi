use std::{borrow::Cow, ffi::{OsStr, OsString}, iter::FusedIterator, ops::Not, path::{self, PathBuf, PrefixComponent}};

use crate::{loc::Loc, scheme::{Scheme, SchemeRef}, url::{Encode, Url, UrlBuf, UrlCow}};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Component<'a> {
	Scheme(SchemeRef<'a>),
	Prefix(PrefixComponent<'a>),
	RootDir,
	CurDir,
	ParentDir,
	Normal(&'a OsStr),
}

impl<'a> From<path::Component<'a>> for Component<'a> {
	fn from(comp: path::Component<'a>) -> Self {
		match comp {
			path::Component::Prefix(p) => Component::Prefix(p),
			path::Component::RootDir => Component::RootDir,
			path::Component::CurDir => Component::CurDir,
			path::Component::ParentDir => Component::ParentDir,
			path::Component::Normal(s) => Component::Normal(s),
		}
	}
}

impl<'a> FromIterator<Component<'a>> for UrlBuf {
	fn from_iter<I: IntoIterator<Item = Component<'a>>>(iter: I) -> Self {
		let mut scheme = Scheme::Regular;
		let mut buf = PathBuf::new();
		iter.into_iter().for_each(|c| match c {
			Component::Scheme(s) => scheme = s.into(),
			Component::Prefix(p) => buf.push(path::Component::Prefix(p)),
			Component::RootDir => buf.push(path::Component::RootDir),
			Component::CurDir => buf.push(path::Component::CurDir),
			Component::ParentDir => buf.push(path::Component::ParentDir),
			Component::Normal(s) => buf.push(path::Component::Normal(s)),
		});

		Self { loc: buf.into(), scheme }
	}
}

impl<'a> FromIterator<Component<'a>> for PathBuf {
	fn from_iter<I: IntoIterator<Item = Component<'a>>>(iter: I) -> Self {
		let mut buf = Self::new();
		iter.into_iter().for_each(|c| match c {
			Component::Scheme(_) => {}
			Component::Prefix(p) => buf.push(path::Component::Prefix(p)),
			Component::RootDir => buf.push(path::Component::RootDir),
			Component::CurDir => buf.push(path::Component::CurDir),
			Component::ParentDir => buf.push(path::Component::ParentDir),
			Component::Normal(s) => buf.push(path::Component::Normal(s)),
		});
		buf
	}
}

// --- Components
#[derive(Clone)]
pub struct Components<'a> {
	inner:          path::Components<'a>,
	loc:            Loc<'a>,
	scheme:         SchemeRef<'a>,
	scheme_yielded: bool,
}

impl<'a> From<Url<'a>> for Components<'a> {
	fn from(value: Url<'a>) -> Self {
		Self {
			inner:          value.loc.as_path().components(),
			loc:            value.loc,
			scheme:         value.scheme,
			scheme_yielded: false,
		}
	}
}

impl<'a> From<&'a UrlBuf> for Components<'a> {
	fn from(value: &'a UrlBuf) -> Self {
		Self {
			inner:          value.loc.components(),
			loc:            value.loc.as_loc(),
			scheme:         value.scheme.as_ref(),
			scheme_yielded: false,
		}
	}
}

impl<'a> From<&'a UrlCow<'a>> for Components<'a> {
	fn from(value: &'a UrlCow<'a>) -> Self { Self::from(value.as_url()) }
}

impl<'a> Components<'a> {
	pub fn os_str(&self) -> Cow<'a, OsStr> {
		let path = self.inner.as_path();
		if !self.scheme.is_virtual() || self.scheme_yielded {
			return path.as_os_str().into();
		}

		let mut s = OsString::from(Encode::new(self.loc, self.scheme).to_string());
		s.reserve_exact(path.as_os_str().len());
		s.push(path);
		s.into()
	}

	pub fn covariant(&self, other: &Self) -> bool {
		match (self.scheme_yielded, other.scheme_yielded) {
			(false, false) => {}
			(true, true) if self.scheme.covariant(other.scheme) => {}
			_ => return false,
		}
		self.inner == other.inner
	}
}

impl<'a> Iterator for Components<'a> {
	type Item = Component<'a>;

	fn next(&mut self) -> Option<Self::Item> {
		if !self.scheme_yielded {
			self.scheme_yielded = true;
			Some(Component::Scheme(self.scheme))
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
		if let Some(comp) = self.inner.next_back() {
			Some(comp.into())
		} else if !self.scheme_yielded {
			self.scheme_yielded = true;
			Some(Component::Scheme(self.scheme))
		} else {
			None
		}
	}
}

impl<'a> FusedIterator for Components<'a> {}

impl<'a> PartialEq for Components<'a> {
	fn eq(&self, other: &Self) -> bool {
		Some(self.scheme).filter(|_| !self.scheme_yielded)
			== Some(other.scheme).filter(|_| !other.scheme_yielded)
			&& self.inner == other.inner
	}
}

// --- Tests
#[cfg(test)]
mod tests {
	use std::path::Path;

	use super::*;
	use crate::pool::InternStr;

	#[test]
	fn test_collect() {
		crate::init_tests();

		let search: UrlBuf = "search://keyword//root/projects/yazi".parse().unwrap();
		assert_eq!(search.loc.uri().as_os_str(), OsStr::new(""));
		assert_eq!(search.scheme, Scheme::Search("keyword".intern()));

		let item = search.join("main.rs");
		assert_eq!(item.loc.uri().as_os_str(), OsStr::new("main.rs"));
		assert_eq!(item.scheme, Scheme::Search("keyword".intern()));

		let u: UrlBuf = item.components().take(4).collect();
		assert_eq!(u.scheme, Scheme::Search("keyword".intern()));
		assert_eq!(u.loc.as_path(), Path::new("/root/projects"));

		let u: UrlBuf = item
			.components()
			.take(5)
			.chain([Component::Normal(OsStr::new("target/release/yazi"))])
			.collect();
		assert_eq!(u.scheme, Scheme::Search("keyword".intern()));
		assert_eq!(u.loc.as_path(), Path::new("/root/projects/yazi/target/release/yazi"));
	}
}
