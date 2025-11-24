use std::path::PrefixComponent;

use anyhow::Result;

use crate::{path::{self, PathDynError}, scheme::SchemeRef, strand::Strand};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Component<'a> {
	Scheme(SchemeRef<'a>),
	Prefix(PrefixComponent<'a>),
	RootDir,
	CurDir,
	ParentDir,
	Normal(Strand<'a>),
}

impl<'a> From<path::Component<'a>> for Component<'a> {
	fn from(value: path::Component<'a>) -> Self {
		match value {
			path::Component::Prefix(p) => Self::Prefix(p),
			path::Component::RootDir => Self::RootDir,
			path::Component::CurDir => Self::CurDir,
			path::Component::ParentDir => Self::ParentDir,
			path::Component::Normal(s) => Self::Normal(s),
		}
	}
}

impl<'a> Component<'a> {
	pub fn downgrade(self) -> Option<path::Component<'a>> {
		Some(match self {
			Self::Scheme(_) => None?,
			Self::Prefix(p) => path::Component::Prefix(p),
			Self::RootDir => path::Component::RootDir,
			Self::CurDir => path::Component::CurDir,
			Self::ParentDir => path::Component::ParentDir,
			Self::Normal(s) => path::Component::Normal(s),
		})
	}
}

// impl<'a> FromIterator<Component<'a>> for Result<UrlBuf> {
// 	fn from_iter<I: IntoIterator<Item = Component<'a>>>(iter: I) -> Self {
// 		let mut buf = PathBuf::new();
// 		let mut scheme = None;
// 		iter.into_iter().for_each(|c| match c {
// 			Component::Scheme(s) => scheme = Some(s),
// 			Component::Prefix(p) => buf.push(path::Component::Prefix(p)),
// 			Component::RootDir => buf.push(path::Component::RootDir),
// 			Component::CurDir => buf.push(path::Component::CurDir),
// 			Component::ParentDir => buf.push(path::Component::ParentDir),
// 			Component::Normal(s) => buf.push(path::Component::Normal(s)),
// 		});

// 		Ok(if let Some(s) = scheme {
// 			UrlCow::try_from((s, PathCow::Owned(PathBufDyn::Os(buf))))?.into_owned()
// 		} else {
// 			buf.into()
// 		})
// 	}
// }

impl<'a> FromIterator<Component<'a>> for Result<std::path::PathBuf, PathDynError> {
	fn from_iter<I: IntoIterator<Item = Component<'a>>>(iter: I) -> Self {
		iter.into_iter().filter_map(|c| c.downgrade()).map(std::path::Component::try_from).collect()
	}
}

impl<'a> FromIterator<Component<'a>> for Result<typed_path::UnixPathBuf, PathDynError> {
	fn from_iter<I: IntoIterator<Item = Component<'a>>>(iter: I) -> Self {
		iter
			.into_iter()
			.filter_map(|c| c.downgrade())
			.map(typed_path::UnixComponent::try_from)
			.collect()
	}
}
