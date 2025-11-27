use std::path::PrefixComponent;

use crate::{path::PathDynError, strand::Strand};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Component<'a> {
	Prefix(PrefixComponent<'a>),
	RootDir,
	CurDir,
	ParentDir,
	Normal(Strand<'a>),
}

impl<'a> Component<'a> {
	pub fn as_normal(&self) -> Option<Strand<'a>> {
		match self {
			Self::Normal(s) => Some(*s),
			_ => None,
		}
	}
}

impl<'a> From<std::path::Component<'a>> for Component<'a> {
	fn from(value: std::path::Component<'a>) -> Self {
		match value {
			std::path::Component::Prefix(p) => Self::Prefix(p),
			std::path::Component::RootDir => Self::RootDir,
			std::path::Component::CurDir => Self::CurDir,
			std::path::Component::ParentDir => Self::ParentDir,
			std::path::Component::Normal(s) => Self::Normal(Strand::Os(s)),
		}
	}
}

impl<'a> From<typed_path::UnixComponent<'a>> for Component<'a> {
	fn from(value: typed_path::UnixComponent<'a>) -> Self {
		match value {
			typed_path::UnixComponent::RootDir => Self::RootDir,
			typed_path::UnixComponent::CurDir => Self::CurDir,
			typed_path::UnixComponent::ParentDir => Self::ParentDir,
			typed_path::UnixComponent::Normal(b) => Self::Normal(Strand::Bytes(b)),
		}
	}
}

impl<'a> TryFrom<Component<'a>> for std::path::Component<'a> {
	type Error = PathDynError;

	fn try_from(value: Component<'a>) -> Result<Self, Self::Error> {
		Ok(match value {
			Component::Prefix(p) => Self::Prefix(p),
			Component::RootDir => Self::RootDir,
			Component::CurDir => Self::CurDir,
			Component::ParentDir => Self::ParentDir,
			Component::Normal(s) => Self::Normal(s.as_os()?),
		})
	}
}

impl<'a> TryFrom<Component<'a>> for typed_path::UnixComponent<'a> {
	type Error = PathDynError;

	fn try_from(value: Component<'a>) -> Result<Self, Self::Error> {
		Ok(match value {
			Component::Prefix(_) => Err(PathDynError::AsUnix)?,
			Component::RootDir => Self::RootDir,
			Component::CurDir => Self::CurDir,
			Component::ParentDir => Self::ParentDir,
			Component::Normal(s) => Self::Normal(s.encoded_bytes()),
		})
	}
}

impl<'a> FromIterator<Component<'a>> for Result<std::path::PathBuf, PathDynError> {
	fn from_iter<I: IntoIterator<Item = Component<'a>>>(iter: I) -> Self {
		iter.into_iter().map(std::path::Component::try_from).collect()
	}
}

impl<'a> FromIterator<Component<'a>> for Result<typed_path::UnixPathBuf, PathDynError> {
	fn from_iter<I: IntoIterator<Item = Component<'a>>>(iter: I) -> Self {
		iter.into_iter().map(typed_path::UnixComponent::try_from).collect()
	}
}
